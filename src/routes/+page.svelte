<script lang="ts">
  import { onMount, tick } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { register, unregister } from "@tauri-apps/plugin-global-shortcut";

  type Note = { id: string; title: string; body: string; updatedAt: number };
  type Folder = { id: string; name: string; notes: Note[] };
  type DragKind = "folder" | "note";
  type DragState = { kind: DragKind; id: string; title: string; sourceIndex: number; targetIndex: number; startX: number; startY: number; clientX: number; clientY: number; active: boolean };

  const storageKey = "quicknotion-data-v1";
  const pinStorageKey = "quicknotion-pinned-v1";
  const shortcutStorageKey = "quicknotion-shortcut-v1";
  const defaultShortcut = "CTRL+SHIFT+SPACE";
  const defaultFolders: Folder[] = [{ id: "work", name: "工作记录", notes: [{ id: "today", title: "今日工作计划", body: "记录今天要完成的事项", updatedAt: Date.now() }] }, { id: "project", name: "项目资料", notes: [] }, { id: "temporary", name: "临时记录", notes: [] }, { id: "ideas", name: "灵感收集", notes: [] }];

  let folders = $state<Folder[]>(loadFolders());
  let currentFolderId = $state<string | null>(null);
  let openedNoteId = $state<string | null>(null);
  let editingFolderId = $state<string | null>(null);
  let editingNoteId = $state<string | null>(null);
  let editingName = $state("");
  let pinned = $state(loadPinned());
  let settingsOpen = $state(false);
  let shortcut = $state(loadShortcut());
  let shortcutDraft = $state(loadShortcut());
  let shortcutEditing = $state(false);
  let shortcutError = $state("");
  let drag = $state<DragState | null>(null);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let resizeTimer: ReturnType<typeof setTimeout> | undefined;
  let baseWindowSize = $state({ width: 380, height: 560 });
  let windowExpanded = $state(false);
  let contentListElement: HTMLElement | null = null;

  function loadFolders(): Folder[] { const stored = localStorage.getItem(storageKey); if (!stored) return defaultFolders; try { return JSON.parse(stored) as Folder[]; } catch { return defaultFolders; } }
  function loadPinned(): boolean { return localStorage.getItem(pinStorageKey) === "true"; }
  function loadShortcut(): string { return localStorage.getItem(shortcutStorageKey) || defaultShortcut; }
  function persistFolders(): void { if (saveTimer) clearTimeout(saveTimer); saveTimer = setTimeout(() => localStorage.setItem(storageKey, JSON.stringify(folders)), 350); }
  function currentFolder(): Folder | undefined { return folders.find((folder) => folder.id === currentFolderId); }
  function createFolder(): void { const folder: Folder = { id: crypto.randomUUID(), name: "新文件夹", notes: [] }; folders = [...folders, folder]; startFolderRename(folder); persistFolders(); }
  function createNote(): void { const folder = currentFolder(); if (!folder) return; const note: Note = { id: crypto.randomUUID(), title: "新笔记", body: "", updatedAt: Date.now() }; folder.notes = [note, ...folder.notes]; openedNoteId = note.id; startNoteRename(note); folders = [...folders]; persistFolders(); }
  function startFolderRename(folder: Folder): void { editingNoteId = null; editingFolderId = folder.id; editingName = folder.name; }
  function startNoteRename(note: Note): void { editingFolderId = null; editingNoteId = note.id; editingName = note.title; }
  function finishRename(): void { const name = editingName.trim(); if (editingFolderId) { const folder = folders.find((item) => item.id === editingFolderId); if (folder && name) folder.name = name; } if (editingNoteId) { const folder = currentFolder(); const note = folder?.notes.find((item) => item.id === editingNoteId); if (note && name) { note.title = name; note.updatedAt = Date.now(); } } editingFolderId = null; editingNoteId = null; editingName = ""; persistFolders(); }
  function cancelRename(): void { editingFolderId = null; editingNoteId = null; editingName = ""; }
  function handleRenameKey(event: KeyboardEvent): void { if (event.key === "Enter") finishRename(); if (event.key === "Escape") cancelRename(); }
  function deleteFolder(folder: Folder): void { folders = folders.filter((item) => item.id !== folder.id); if (currentFolderId === folder.id) currentFolderId = null; persistFolders(); }
  async function deleteNote(note: Note): Promise<void> { const folder = currentFolder(); if (!folder) return; folder.notes = folder.notes.filter((item) => item.id !== note.id); if (openedNoteId === note.id) { openedNoteId = null; await restoreWindowSize(); } folders = [...folders]; persistFolders(); }
  async function readWindowSize(): Promise<void> { const currentWindow = getCurrentWindow(); const size = await currentWindow.innerSize(); baseWindowSize = { width: size.width, height: size.height }; }
  async function setWindowSize(width: number, height: number): Promise<void> { const size = { width, height }; await invoke("resize_main_window", size); }
  async function restoreWindowSize(): Promise<void> { if (!windowExpanded) return; windowExpanded = false; await setWindowSize(baseWindowSize.width, baseWindowSize.height); }
  async function toggleNote(id: string): Promise<void> { if (openedNoteId === id) { openedNoteId = null; await restoreWindowSize(); return; } if (!openedNoteId) await readWindowSize(); openedNoteId = id; windowExpanded = true; await tick(); scheduleResize(); }
  function initializeEditor(node: HTMLElement, note: Note) {
    node.textContent = note.body;
    let composing = false;
    function saveContent(): void {
      if (composing) return;
      const body = node.innerText;
      if (note.body === body) return;
      note.body = body;
      note.updatedAt = Date.now();
      persistFolders();
      scheduleResize();
    }
    function beginComposition(): void { composing = true; }
    function endComposition(): void { composing = false; saveContent(); }
    node.addEventListener("input", saveContent);
    node.addEventListener("compositionstart", beginComposition);
    node.addEventListener("compositionend", endComposition);
    node.addEventListener("blur", endComposition);
    function destroy(): void {
      node.removeEventListener("input", saveContent);
      node.removeEventListener("compositionstart", beginComposition);
      node.removeEventListener("compositionend", endComposition);
      node.removeEventListener("blur", endComposition);
    }
    return { destroy };
  }
  async function applyPinState(enabled: boolean): Promise<void> { await invoke("set_window_always_on_top", { enabled }); }
  async function togglePin(): Promise<void> { const next = !pinned; try { await applyPinState(next); pinned = next; localStorage.setItem(pinStorageKey, String(next)); } catch { pinned = !next; } }
  async function hideWindow(): Promise<void> { await invoke("hide_main_window"); }
  async function startWindowDrag(event: MouseEvent): Promise<void> { if (event.button !== 0 || (event.target as HTMLElement).closest("button,input")) return; await invoke("start_window_drag"); }

  function dragStart(event: PointerEvent, kind: DragKind, id: string, title: string, index: number): void { if (event.button !== 0 || settingsOpen) return; event.preventDefault(); drag = { kind, id, title, sourceIndex: index, targetIndex: index, startX: event.clientX, startY: event.clientY, clientX: event.clientX, clientY: event.clientY, active: false }; document.body.classList.add("dragging-list"); window.addEventListener("pointermove", dragMove); window.addEventListener("pointerup", dragEnd); window.addEventListener("pointercancel", dragCancel); }
  function dragMove(event: PointerEvent): void { if (!drag) return; const distance = Math.abs(event.clientX - drag.startX) + Math.abs(event.clientY - drag.startY); if (!drag.active && distance < 6) return; const next = { ...drag, active: true, clientX: event.clientX, clientY: event.clientY }; const row = document.elementFromPoint(event.clientX, event.clientY)?.closest<HTMLElement>(`[data-sort-kind="${drag.kind}"][data-sort-index]`); const list = document.querySelector<HTMLElement>(`[data-sort-list="${drag.kind}"]`); if (row) { const index = Number(row.dataset.sortIndex); const rect = row.getBoundingClientRect(); next.targetIndex = event.clientY < rect.top + rect.height / 2 ? index : index + 1; } else if (list) { const rect = list.getBoundingClientRect(); if (event.clientY < rect.top) next.targetIndex = 0; if (event.clientY > rect.bottom) next.targetIndex = drag.kind === "folder" ? folders.length : currentFolder()?.notes.length ?? 0; } drag = next; }
  function dragEnd(): void { if (!drag) return; const currentDrag = drag; removeDragListeners(); if (currentDrag.active) { const finalIndex = currentDrag.targetIndex > currentDrag.sourceIndex ? currentDrag.targetIndex - 1 : currentDrag.targetIndex; moveItem(currentDrag.kind, currentDrag.sourceIndex, finalIndex); persistFolders(); } drag = null; }
  function dragCancel(): void { removeDragListeners(); drag = null; }
  function removeDragListeners(): void { window.removeEventListener("pointermove", dragMove); window.removeEventListener("pointerup", dragEnd); window.removeEventListener("pointercancel", dragCancel); document.body.classList.remove("dragging-list"); }
  function moveItem(kind: DragKind, from: number, to: number): void { if (from === to || to < 0) return; if (kind === "folder") { const items = [...folders]; const item = items.splice(from, 1)[0]; items.splice(to, 0, item); folders = items; return; } const folder = currentFolder(); if (!folder) return; const items = [...folder.notes]; const item = items.splice(from, 1)[0]; items.splice(to, 0, item); folder.notes = items; folders = [...folders]; }

  function scheduleResize(): void { if (resizeTimer) clearTimeout(resizeTimer); resizeTimer = setTimeout(resizeForContent, 50); }
  async function resizeForContent(): Promise<void> {
    if (!openedNoteId || settingsOpen || !windowExpanded || !contentListElement) return;
    const expandedItem = contentListElement.querySelector<HTMLElement>(`article[data-note-id="${openedNoteId}"]`);
    if (!expandedItem) return;
    const nextItem = expandedItem.nextElementSibling as HTMLElement | null;
    const nextTitle = nextItem?.querySelector<HTMLElement>(".note-row");
    const visibleBoundary = nextTitle || expandedItem;
    const listTop = contentListElement.getBoundingClientRect().top;
    const boundaryBottom = visibleBoundary.getBoundingClientRect().bottom;
    const bottomPadding = 20;
    const titlebar = document.querySelector<HTMLElement>(".titlebar");
    const titlebarHeight = titlebar?.offsetHeight || 44;
    const requiredHeight = Math.ceil(titlebarHeight + boundaryBottom - listTop + contentListElement.scrollTop + bottomPadding);
    const currentWindow = getCurrentWindow();
    const currentSize = await currentWindow.innerSize();
    if (currentSize.height >= requiredHeight) return;
    await setWindowSize(baseWindowSize.width, Math.max(400, requiredHeight));
  }
  async function openSettings(): Promise<void> { settingsOpen = true; currentFolderId = null; openedNoteId = null; await restoreWindowSize(); }
  async function closeSettings(): Promise<void> { settingsOpen = false; await tick(); }
  function keyLabel(event: KeyboardEvent): string { const parts: string[] = []; if (event.ctrlKey) parts.push("CTRL"); if (event.altKey) parts.push("ALT"); if (event.shiftKey) parts.push("SHIFT"); if (event.metaKey) parts.push("META"); const key = event.key.toUpperCase(); if (!["CONTROL", "ALT", "SHIFT", "META"].includes(key)) parts.push(key === " " ? "SPACE" : key); return parts.join("+"); }
  function captureShortcut(event: KeyboardEvent): void { event.preventDefault(); if (event.key === "Escape") { shortcutEditing = false; shortcutDraft = shortcut; shortcutError = ""; return; } const value = keyLabel(event); if (!value || ![event.ctrlKey, event.altKey, event.shiftKey, event.metaKey].some(Boolean)) { shortcutError = "请至少包含一个修饰键"; return; } shortcutDraft = value; shortcutEditing = false; void saveShortcut(value); }
  async function saveShortcut(value: string): Promise<void> { const previous = shortcut; try { await register(value, (event) => { if (event.state === "Pressed") void invoke("toggle_main_window"); }); if (previous !== value) await unregister(previous); shortcut = value; localStorage.setItem(shortcutStorageKey, value); shortcutError = ""; } catch { shortcutError = "快捷键注册失败，可能与其他应用冲突"; shortcutDraft = previous; } }
  async function resetShortcut(): Promise<void> { await saveShortcut(defaultShortcut); }

  onMount(() => { void applyPinState(pinned); void saveShortcut(shortcut); const removeSettings = listen("open-settings", openSettings); const keydown = (event: KeyboardEvent) => { if (event.key === "Escape" && drag) dragCancel(); }; window.addEventListener("keydown", keydown); return () => { removeDragListeners(); window.removeEventListener("keydown", keydown); removeSettings.then((remove) => remove()); }; });
</script>

<svelte:head><title>QuickNotion</title></svelte:head>
<main class="app-shell">
  <header class="titlebar" role="banner" onmousedown={startWindowDrag}>
    <button class="icon-button" type="button" aria-label="返回" onclick={() => settingsOpen ? closeSettings() : (currentFolderId = null)} disabled={!settingsOpen && !currentFolderId}>‹</button>
    <strong>{settingsOpen ? "设置" : currentFolder()?.name ?? "便签"}</strong><span class="spacer"></span>
    {#if !settingsOpen}<button class="icon-button" type="button" aria-label="新建" onclick={currentFolderId ? createNote : createFolder}>＋</button><button class:pinned class="icon-button pin-button" type="button" aria-label="置顶" onclick={togglePin}>◆</button>{/if}
    <button class="icon-button" type="button" aria-label="隐藏窗口" onclick={hideWindow}>×</button>
  </header>
  {#if settingsOpen}
    <section class="settings"><h2>设置</h2><div class="setting-row"><div><strong>显示 / 隐藏便签</strong><small>使用全局快捷键唤起应用</small></div>{#if shortcutEditing}<input class="shortcut-input" value={shortcutDraft} onkeydown={captureShortcut} onblur={() => (shortcutEditing = false)} autofocus />{:else}<button class="shortcut-value" type="button" onclick={() => (shortcutEditing = true)}>{shortcut}</button>{/if}</div>{#if shortcutError}<div class="error">{shortcutError}</div>{/if}<button class="reset-button" type="button" onclick={resetShortcut}>恢复默认快捷键</button></section>
  {:else}
    <section class="content-list" role="list" bind:this={contentListElement} data-sort-list={currentFolderId ? "note" : "folder"}>
      {#if !currentFolderId}
        {#if folders.length === 0}<div class="empty">还没有文件夹</div>{:else}{#each folders as folder, index (folder.id)}{#if drag?.kind === "folder" && drag.active && drag.targetIndex === index}<div class="drop-indicator"></div>{/if}<article class:dragging-row={drag?.kind === "folder" && drag?.active && drag?.id === folder.id} class="list-row" role="listitem" data-sort-kind="folder" data-sort-index={index}><button class="drag-handle" type="button" aria-label={`拖动${folder.name}`} onpointerdown={(event) => dragStart(event, "folder", folder.id, folder.name, index)}>⋮⋮</button>{#if editingFolderId === folder.id}<input class="inline-name" bind:value={editingName} onkeydown={handleRenameKey} onblur={finishRename} autofocus />{:else}<button class="row-main" type="button" onclick={() => (currentFolderId = folder.id)}><span class="folder-icon">▣</span><span>{folder.name}</span></button>{/if}<button class="rename-button" type="button" aria-label="重命名文件夹" onclick={() => startFolderRename(folder)}>✎</button><button class="delete-button" type="button" aria-label="删除文件夹" onclick={() => deleteFolder(folder)}>×</button></article>{/each}{#if drag?.kind === "folder" && drag.active && drag.targetIndex === folders.length}<div class="drop-indicator"></div>{/if}{/if}
      {:else}
        {#if !(currentFolder()?.notes.length ?? 0)}<div class="empty">还没有笔记，点击右上角 ＋ 新建</div>{:else}{#each currentFolder()?.notes ?? [] as note, index (note.id)}{#if drag?.kind === "note" && drag.active && drag.targetIndex === index}<div class="drop-indicator"></div>{/if}<article class:dragging-row={drag?.kind === "note" && drag?.active && drag?.id === note.id} class:expanded={openedNoteId === note.id} class="note-block" role="listitem" data-note-id={note.id} data-sort-kind="note" data-sort-index={index}><div class="list-row note-row"><button class="drag-handle" type="button" aria-label={`拖动${note.title}`} onpointerdown={(event) => dragStart(event, "note", note.id, note.title, index)}>⋮⋮</button>{#if editingNoteId === note.id}<input class="inline-name" bind:value={editingName} onkeydown={handleRenameKey} onblur={finishRename} autofocus />{:else}<button class="row-main" type="button" onclick={() => toggleNote(note.id)}><span>{note.title}</span></button>{/if}<button class="rename-button" type="button" aria-label="重命名笔记" onclick={() => startNoteRename(note)}>✎</button><button class="delete-button" type="button" aria-label="删除笔记" onclick={() => deleteNote(note)}>×</button></div>{#if openedNoteId === note.id}<div class="editor-panel"><div class="editor" contenteditable="true" role="textbox" aria-label="笔记正文" use:initializeEditor={note}></div></div>{/if}</article>{/each}{#if drag?.kind === "note" && drag.active && drag.targetIndex === (currentFolder()?.notes.length ?? 0)}<div class="drop-indicator"></div>{/if}{/if}
      {/if}
    </section>
  {/if}
  <div class:visible={drag?.active} class="drag-ghost" style:transform={`translate(${(drag?.clientX ?? 0) + 12}px, ${(drag?.clientY ?? 0) + 12}px)`}>{drag?.kind === "folder" ? "▣" : "▤"}<span>{drag?.title}</span></div>
</main>

<style>
  :global(*){box-sizing:border-box}:global(html),:global(body){margin:0;min-width:320px;min-height:100%;background:#171b20;color:#e8ebef;font-family:"Segoe UI Variable","Microsoft YaHei UI",sans-serif}:global(body){overflow:hidden}:global(body.dragging-list){user-select:none;cursor:grabbing}button{color:inherit;font:inherit;cursor:pointer}.app-shell{width:100%;min-width:0;min-height:100vh;display:flex;flex-direction:column;background:#171b20}.titlebar{width:100%;height:44px;display:flex;align-items:center;gap:5px;padding:0 11px;background:#1c2026;border-bottom:1px solid #2b3138;user-select:none}.titlebar strong{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:13px}.spacer{flex:1}.icon-button,.rename-button,.delete-button{width:31px;height:30px;border:0;border-radius:6px;background:transparent;color:#b7bfcb;display:inline-grid;place-items:center}.icon-button{font-size:18px}.icon-button:hover,.rename-button:hover,.delete-button:hover{background:#2b323b;color:#fff}.pin-button.pinned{background:#34456c;color:#baccff}.content-list{width:100%;min-width:0;flex:1;overflow:auto;padding:13px 12px 20px;scrollbar-color:#424b55 transparent}.list-row{min-width:0;min-height:49px;display:flex;align-items:center;margin-bottom:8px;border:1px solid transparent;border-radius:7px;background:#20252c}.list-row:hover{background:#272d36}.row-main{flex:1;min-width:0;display:flex;align-items:center;gap:8px;padding:12px;border:0;background:transparent;color:inherit;text-align:left;overflow:hidden}.row-main span:last-child{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}.folder-icon{color:#8996ab;font-size:17px}.drag-handle{width:30px;align-self:stretch;border:0;background:transparent;color:#687483;cursor:grab;letter-spacing:-4px}.drag-handle:hover{color:#baccff;background:#303b4b}.drag-handle:active{cursor:grabbing}.rename-button,.delete-button{opacity:0;font-size:17px}.list-row:hover .rename-button,.list-row:hover .delete-button,.rename-button:focus,.delete-button:focus{opacity:1}.delete-button{color:#d98989}.inline-name{flex:1;min-width:0;margin:7px 5px;padding:7px 8px;border:1px solid #7289da;border-radius:5px;outline:none;background:#14191f;color:#fff;font:inherit}.note-block{min-width:0}.note-block.expanded .list-row{margin-bottom:0;border-color:#3a4555;border-radius:7px 7px 0 0;background:#242b35}.editor-panel{width:100%;min-width:0;padding:13px;min-height:142px;border:1px solid #3a4555;border-top:0;border-radius:0 0 8px 8px;background:#222831;overflow:hidden}.editor{width:100%;min-width:0;min-height:95px;outline:none;line-height:1.7;white-space:pre-wrap;overflow-wrap:anywhere;word-break:break-word}.editor:empty:before{content:"直接输入文字…";color:#777f8c}.empty{padding:62px 12px;color:#8c96a4;text-align:center;font-size:12px}.settings{width:100%;min-width:0;padding:20px 15px}.settings h2{margin:0 0 20px;font-size:18px}.setting-row{display:flex;align-items:center;gap:12px;padding:15px;border-radius:8px;background:#20252c}.setting-row div{flex:1}.setting-row small{display:block;margin-top:6px;color:#8d97a5;font-size:11px}.shortcut-value,.shortcut-input{width:150px;padding:8px;border:1px solid #465365;border-radius:6px;background:#14191f;color:#e8ebef;text-align:center}.reset-button{margin-top:15px;padding:8px 12px;border:0;border-radius:6px;background:#303b4b;color:#dce5f2}.error{margin-top:10px;color:#e99a9a;font-size:12px}.drop-indicator{height:4px;margin:2px 7px 6px;border-radius:3px;background:#76a3ff;box-shadow:0 0 0 1px #36558e,0 0 12px #648fff}.dragging-row{opacity:.32;transform:scale(.98);border-color:#6989c9;box-shadow:inset 0 0 0 1px #526fa8}.drag-ghost{position:fixed;z-index:1000;display:none;align-items:center;gap:8px;width:calc(100% - 32px);max-width:350px;padding:12px;border:1px solid #789cff;border-radius:8px;background:#29364c;color:#fff;box-shadow:0 12px 28px #0009;pointer-events:none;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.drag-ghost.visible{display:flex}.drag-ghost span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
</style>
