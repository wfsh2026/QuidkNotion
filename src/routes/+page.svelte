<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";

  type Note = { id: string; title: string; body: string; updatedAt: number };
  type Folder = { id: string; name: string; notes: Note[] };

  const storageKey = "quicknotion-data-v1";
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
  let saveState = $state("已保存");
  let saveTimer: ReturnType<typeof setTimeout> | undefined;

  function loadFolders(): Folder[] {
    const stored = localStorage.getItem(storageKey);
    if (!stored) return defaultFolders;
    try { return JSON.parse(stored) as Folder[]; } catch { return defaultFolders; }
  }

  function persistFolders(): void {
    saveState = "保存中";
    if (saveTimer) clearTimeout(saveTimer);
    saveTimer = setTimeout(() => {
      try { localStorage.setItem(storageKey, JSON.stringify(folders)); saveState = "已保存"; } catch { saveState = "保存失败"; }
    }, 500);
  }

  function currentFolder(): Folder | undefined { return folders.find((folder) => folder.id === currentFolderId); }

  function createFolder(): void {
    const name = window.prompt("文件夹名称", "新文件夹");
    if (!name?.trim()) return;
    folders.push({ id: crypto.randomUUID(), name: name.trim(), notes: [] });
    persistFolders();
  }

  function createNote(): void {
    const folder = currentFolder();
    if (!folder) return;
    const note: Note = { id: crypto.randomUUID(), title: "新笔记", body: "", updatedAt: Date.now() };
    folder.notes.unshift(note);
    openedNoteId = note.id;
    persistFolders();
  }

  function renameFolder(folder: Folder): void {
    const name = window.prompt("重命名文件夹", folder.name);
    if (name?.trim()) { folder.name = name.trim(); persistFolders(); }
  }

  function renameNote(note: Note): void {
    const name = window.prompt("重命名笔记", note.title);
    if (name?.trim()) { note.title = name.trim(); note.updatedAt = Date.now(); persistFolders(); }
  }

  function deleteFolder(folder: Folder): void {
    const confirmed = window.confirm("删除“" + folder.name + "”？");
    if (!confirmed) return;
    folders = folders.filter((item) => item.id !== folder.id);
    persistFolders();
  }

  function deleteNote(note: Note): void {
    const folder = currentFolder();
    if (!folder) return;
    const confirmed = window.confirm("删除“" + note.title + "”？");
    if (!confirmed) return;
    folder.notes = folder.notes.filter((item) => item.id !== note.id);
    openedNoteId = null;
    persistFolders();
  }

  function toggleNote(noteId: string): void { openedNoteId = openedNoteId === noteId ? null : noteId; }

  function updateNote(note: Note, event: Event): void {
    const target = event.currentTarget as HTMLElement;
    note.body = target.innerText;
    note.updatedAt = Date.now();
    persistFolders();
  }

  function hideWindow(): void { appWindow.hide(); }
</script>

<svelte:head><title>QuickNotion</title></svelte:head>

<main class="app-shell">
  <header class="titlebar" data-tauri-drag-region>
    <button class="icon-button" aria-label="返回" onclick={() => (currentFolderId = null)} disabled={!currentFolderId}>‹</button>
    <strong>{currentFolder()?.name ?? "便签"}</strong>
    <span class="spacer"></span>
    <button class="icon-button" aria-label="新建" onclick={currentFolderId ? createNote : createFolder}>＋</button>
    <button class="icon-button" aria-label="隐藏窗口" onclick={hideWindow}>×</button>
  </header>

  <section class="content-list">
    {#if !currentFolderId}
      {#if folders.length === 0}
        <div class="empty">还没有文件夹</div>
      {:else}
        {#each folders as folder (folder.id)}
          <article class="list-row">
            <button class="row-main" onclick={() => (currentFolderId = folder.id)}><span class="folder-icon">▣</span><span>{folder.name}</span><span class="chevron">›</span></button>
            <button class="more-button" aria-label="文件夹菜单" onclick={() => renameFolder(folder)}>···</button>
            <button class="delete-button" aria-label="删除文件夹" onclick={() => deleteFolder(folder)}>×</button>
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
              <button class="row-main" onclick={() => toggleNote(note.id)}><span>{note.title}</span><span class="chevron">{openedNoteId === note.id ? "⌄" : "›"}</span></button>
              <button class="more-button" aria-label="笔记菜单" onclick={() => renameNote(note)}>···</button>
              <button class="delete-button" aria-label="删除笔记" onclick={() => deleteNote(note)}>×</button>
            </div>
            {#if openedNoteId === note.id}
              <div class="editor-panel">
                <div class="editor" contenteditable="true" role="textbox" aria-label="笔记正文" oninput={(event) => updateNote(note, event)}>{note.body}</div>
                <div class="editor-foot"><span>{saveState}</span><span>文本自动保存</span></div>
              </div>
            {/if}
          </article>
        {/each}
      {/if}
    {/if}
  </section>

  <footer class="statusbar"><span>本地保存</span><span>{saveState}</span></footer>
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
  .icon-button, .more-button, .delete-button { width: 31px; height: 30px; border: 0; border-radius: 6px; background: transparent; color: #b7bfcb; display: inline-grid; place-items: center; }
  .icon-button { font-size: 20px; }
  .icon-button:hover, .more-button:hover, .delete-button:hover { background: #2b323b; color: #fff; }
  .icon-button:disabled { opacity: .3; cursor: default; }
  .content-list { flex: 1; overflow: auto; padding: 13px 12px 20px; scrollbar-color: #424b55 transparent; }
  .list-row { min-height: 49px; display: flex; align-items: center; margin-bottom: 8px; border: 1px solid transparent; border-radius: 7px; background: #20252c; }
  .list-row:hover { background: #272d36; }
  .row-main { flex: 1; min-width: 0; display: flex; align-items: center; gap: 8px; padding: 12px 5px 12px 12px; border: 0; background: transparent; color: inherit; text-align: left; overflow: hidden; }
  .row-main span:nth-child(2) { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .folder-icon { color: #8996ab; font-size: 17px; }
  .chevron { margin-left: auto; color: #84909c; font-size: 18px; }
  .more-button, .delete-button { opacity: 0; font-size: 17px; }
  .list-row:hover .more-button, .list-row:hover .delete-button { opacity: 1; }
  .delete-button { color: #d98989; }
  .note-block.expanded .list-row { margin-bottom: 0; border-color: #3a4555; border-radius: 7px 7px 0 0; background: #242b35; }
  .editor-panel { padding: 13px; min-height: 142px; max-height: 350px; overflow: auto; border: 1px solid #3a4555; border-top: 0; border-radius: 0 0 8px 8px; background: #222831; }
  .editor { min-height: 95px; outline: none; line-height: 1.7; white-space: pre-wrap; overflow-wrap: anywhere; }
  .editor:empty:before { content: "直接输入文字…"; color: #777f8c; }
  .editor-foot, .statusbar { display: flex; justify-content: space-between; gap: 10px; color: #818c9b; font-size: 11px; }
  .editor-foot { margin-top: 10px; }
  .statusbar { padding: 6px 14px 10px; }
  .empty { padding: 62px 12px; color: #8c96a4; text-align: center; font-size: 12px; }
</style>
