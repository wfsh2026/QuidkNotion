<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  type Note = { id: string; title: string; body: string; updatedAt: number };
  type Folder = { id: string; name: string; notes: Note[] };

  const storageKey = "quicknotion-data-v1";
  const pinStorageKey = "quicknotion-pinned-v1";
  const appWindow = getCurrentWindow();
  const defaultFolders: Folder[] = [
    { id: "work", name: "工作记录", notes: [{ id: "today", title: "今日工作计划", body: "记录今天要完成的事项", updatedAt: Date.now() }] },
    { id: "project", name: "项目资料", notes: [] },
    { id: "temporary", name: "临时记录", notes: [] },
    { id: "ideas", name: "灵感收集", notes: [] }
  ];

  let folders = $state<Folder[]>(loadFolders());
  let currentFolderId = $state<string | null>(null);
  let openedNoteId = $state<string | null>(null);
  let editingFolderId = $state<string | null>(null);
  let editingNoteId = $state<string | null>(null);
  let editingName = $state("");
  let pinned = $state(loadPinned());
  void appWindow.setAlwaysOnTop(pinned);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  function loadFolders(): Folder[] {
    const stored = localStorage.getItem(storageKey);
    if (!stored) return defaultFolders;
    try { return JSON.parse(stored) as Folder[]; } catch { return defaultFolders; }
  }

  function loadPinned(): boolean { return localStorage.getItem(pinStorageKey) === "true"; }

  function persistFolders(): void {
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => localStorage.setItem(storageKey, JSON.stringify(folders)), 500);
  }

  function currentFolder(): Folder | undefined { return folders.find((folder) => folder.id === currentFolderId); }

  function createFolder(): void {
    const folder: Folder = { id: crypto.randomUUID(), name: "新文件夹", notes: [] };
    folders.push(folder);
    startFolderRename(folder);
    persistFolders();
  }

  function createNote(): void {
    const folder = currentFolder();
    if (!folder) return;
    const note: Note = { id: crypto.randomUUID(), title: "新笔记", body: "", updatedAt: Date.now() };
    folder.notes.unshift(note);
    openedNoteId = note.id;
    startNoteRename(note);
    persistFolders();
  }

  function startFolderRename(folder: Folder): void {
    editingNoteId = null;
    editingFolderId = folder.id;
    editingName = folder.name;
  }

  function startNoteRename(note: Note): void {
    editingFolderId = null;
    editingNoteId = note.id;
    editingName = note.title;
  }

  function finishRename(): void {
    const name = editingName.trim();
    if (editingFolderId) {
      const folder = folders.find((item) => item.id === editingFolderId);
      if (folder && name) folder.name = name;
    }
    if (editingNoteId) {
      const folder = currentFolder();
      const note = folder?.notes.find((item) => item.id === editingNoteId);
      if (note && name) { note.title = name; note.updatedAt = Date.now(); }
    }
    editingFolderId = null;
    editingNoteId = null;
    editingName = "";
    persistFolders();
  }

  function cancelRename(): void {
    editingFolderId = null;
    editingNoteId = null;
    editingName = "";
  }

  function handleRenameKey(event: KeyboardEvent): void {
    if (event.key === "Enter") finishRename();
    if (event.key === "Escape") cancelRename();
  }

  function deleteFolder(folder: Folder): void {
    folders = folders.filter((item) => item.id !== folder.id);
    if (currentFolderId === folder.id) currentFolderId = null;
    persistFolders();
  }

  function deleteNote(note: Note): void {
    const folder = currentFolder();
    if (!folder) return;
    folder.notes = folder.notes.filter((item) => item.id !== note.id);
    if (openedNoteId === note.id) openedNoteId = null;
    persistFolders();
  }

  function toggleNote(noteId: string): void { openedNoteId = openedNoteId === noteId ? null : noteId; }

  function updateNote(note: Note, event: Event): void {
    const target = event.currentTarget as HTMLElement;
    note.body = target.innerText;
    note.updatedAt = Date.now();
    persistFolders();
  }

  async function togglePin(): Promise<void> {
    pinned = !pinned;
    localStorage.setItem(pinStorageKey, String(pinned));
    await appWindow.setAlwaysOnTop(pinned);
  }

  async function hideWindow(): Promise<void> { await appWindow.hide(); }
</script>

<svelte:head><title>QuickNotion</title></svelte:head>

<main class="app-shell">
  <header class="titlebar" data-tauri-drag-region>
    <button class="icon-button" type="button" aria-label="返回" onclick={() => (currentFolderId = null)} disabled={!currentFolderId}>‹</button>
    <strong>{currentFolder()?.name ?? "便签"}</strong>
    <span class="spacer"></span>
    <button class="icon-button" type="button" aria-label="新建" onclick={currentFolderId ? createNote : createFolder}>＋</button>
    <button class:pinned class="icon-button pin-button" type="button" aria-label={pinned ? "取消置顶" : "置顶窗口"} onclick={togglePin}>◆</button>
    <button class="icon-button" type="button" aria-label="隐藏窗口" onclick={hideWindow}>×</button>
  </header>

  <section class="content-list">
    {#if !currentFolderId}
      {#if folders.length === 0}
        <div class="empty">还没有文件夹</div>
      {:else}
        {#each folders as folder (folder.id)}
          <article class="list-row">
            {#if editingFolderId === folder.id}
              <input class="inline-name" aria-label="文件夹名称" bind:value={editingName} onkeydown={handleRenameKey} onblur={finishRename} autofocus />
            {:else}
              <button class="row-main" type="button" onclick={() => (currentFolderId = folder.id)}><span class="folder-icon">▣</span><span>{folder.name}</span></button>
            {/if}
            <button class="rename-button" type="button" aria-label="重命名文件夹" onclick={() => startFolderRename(folder)}>✎</button>
            <button class="delete-button" type="button" aria-label="删除文件夹" onclick={() => deleteFolder(folder)}>×</button>
          </article>
        {/each}
      {/if}
    {:else}
      {#if !(currentFolder()?.notes.length ?? 0)}
        <div class="empty">还没有笔记，点击右上角 ＋ 新建</div>
      {:else}
        {#each currentFolder()?.notes ?? [] as note (note.id)}
          <article class:expanded={openedNoteId === note.id} class="note-block">
            <div class="list-row note-row">
              {#if editingNoteId === note.id}
                <input class="inline-name" aria-label="笔记标题" bind:value={editingName} onkeydown={handleRenameKey} onblur={finishRename} autofocus />
              {:else}
                <button class="row-main" type="button" onclick={() => toggleNote(note.id)}><span>{note.title}</span></button>
              {/if}
              <button class="rename-button" type="button" aria-label="重命名笔记" onclick={() => startNoteRename(note)}>✎</button>
              <button class="delete-button" type="button" aria-label="删除笔记" onclick={() => deleteNote(note)}>×</button>
            </div>
            {#if openedNoteId === note.id}
              <div class="editor-panel"><div class="editor" contenteditable="true" role="textbox" aria-label="笔记正文" oninput={(event) => updateNote(note, event)}>{note.body}</div></div>
            {/if}
          </article>
        {/each}
      {/if}
    {/if}
  </section>
</main>

<style>
  :global(*) { box-sizing: border-box; }
  :global(html), :global(body) { margin: 0; min-width: 320px; min-height: 100%; background: #171b20; color: #e8ebef; font-family: "Segoe UI Variable", "Microsoft YaHei UI", sans-serif; }
  :global(body) { overflow: hidden; }
  button { color: inherit; font: inherit; cursor: pointer; }
  .app-shell { min-height: 100vh; display: flex; flex-direction: column; background: #171b20; }
  .titlebar { height: 44px; display: flex; align-items: center; gap: 5px; padding: 0 11px; background: #1c2026; border-bottom: 1px solid #2b3138; user-select: none; }
  .titlebar strong { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 13px; font-weight: 600; }
  .spacer { flex: 1; }
  .icon-button, .rename-button, .delete-button { width: 31px; height: 30px; border: 0; border-radius: 6px; background: transparent; color: #b7bfcb; display: inline-grid; place-items: center; }
  .icon-button { font-size: 18px; }
  .icon-button:hover, .rename-button:hover, .delete-button:hover { background: #2b323b; color: #fff; }
  .icon-button:disabled { opacity: .3; cursor: default; }
  .pin-button.pinned { background: #34456c; color: #baccff; }
  .content-list { flex: 1; overflow: auto; padding: 13px 12px 20px; scrollbar-color: #424b55 transparent; }
  .list-row { min-height: 49px; display: flex; align-items: center; margin-bottom: 8px; border: 1px solid transparent; border-radius: 7px; background: #20252c; }
  .list-row:hover { background: #272d36; }
  .row-main { flex: 1; min-width: 0; display: flex; align-items: center; gap: 8px; padding: 12px; border: 0; background: transparent; color: inherit; text-align: left; overflow: hidden; }
  .row-main span:last-child { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .folder-icon { color: #8996ab; font-size: 17px; }
  .rename-button, .delete-button { opacity: 0; font-size: 17px; }
  .list-row:hover .rename-button, .list-row:hover .delete-button, .rename-button:focus, .delete-button:focus { opacity: 1; }
  .delete-button { color: #d98989; }
  .inline-name { flex: 1; min-width: 0; margin: 7px 5px 7px 12px; padding: 7px 8px; border: 1px solid #7289da; border-radius: 5px; outline: none; background: #14191f; color: #fff; font: inherit; }
  .note-block.expanded .list-row { margin-bottom: 0; border-color: #3a4555; border-radius: 7px 7px 0 0; background: #242b35; }
  .editor-panel { padding: 13px; min-height: 142px; max-height: 350px; overflow: auto; border: 1px solid #3a4555; border-top: 0; border-radius: 0 0 8px 8px; background: #222831; }
  .editor { min-height: 95px; outline: none; line-height: 1.7; white-space: pre-wrap; overflow-wrap: anywhere; }
  .editor:empty:before { content: "直接输入文字…"; color: #777f8c; }
  .empty { padding: 62px 12px; color: #8c96a4; text-align: center; font-size: 12px; }
</style>


