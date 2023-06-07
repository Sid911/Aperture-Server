import { Component, For, createSignal, onMount } from "solid-js";
import { invoke } from "@tauri-apps/api/tauri";
import { FileEntry } from '@tauri-apps/api/fs'

import { open } from "@tauri-apps/api/dialog"
import { VsAdd, VsKebabVertical } from 'solid-icons/vs'
import { RiDocumentFoldersFill } from 'solid-icons/ri'
import { readDir, BaseDirectory } from '@tauri-apps/api/fs';
import FolderDropdown from "./FolderDropdown";

async function handleFolderSelection() {
  let selected = await open({ directory: true })
  if (Array.isArray(selected)) {
    // user selected multiple directories
  } else if (selected === null) {
    // user cancelled the selection
  } else {
    // user selected a single directory
  }
}

export const FolderSelector: Component<{}> = (props) => {
  return <div class="dark:bg-neutral-800 rounded-lg p-3 flex flex-col">
    <div class="flex flex-row items-center">
    <span class="p-2 dark:text-neutral-300">Folders:</span> <span><RiDocumentFoldersFill class="m-1" size={18}/></span>
    </div>
    <ExistingFolders />
    <hr class="border-transparent border-2 my-2" />
    <NewFolderButton />
  </div>;
};


const ExistingFolders: Component<{}> = (props) => {
  const [dirs, setDirs] = createSignal<FileEntry[]>([]);

  onMount(async () => {
    // initTE({ Dropdown, Ripple });

    try {
      const subfolders = await readDir('Aperture', { dir: BaseDirectory.Document, recursive: false });;
      const folderEntries: FileEntry[] = subfolders;
      console.log(folderEntries);
      setDirs(folderEntries);
    } catch (error) {   
      console.error("Failed to fetch subfolders:", error);
    }
  });

  return <For each={dirs()}>{
    (dir, i) =>
      <Folder entry={dir} />
  }
  </For>;
};

const Folder: Component<{ entry: FileEntry }> = (props) => {

  return <div class="dark:bg-neutral-900 rounded-lg p-3 flex flex-row justify-between content-center 
  shadow-md hover:shadow-lg hover:shadow-black shadow-black my-1 hover:z-10">
    <p><span>{props.entry.name}</span> <span class="text-xs dark:text-neutral-500">{props.entry.path}</span></p>
    <FolderDropdown/>
  </div>;
};


const NewFolderButton: Component<{}> = (props) => {

  return <button class="rounded-lg p-3 flex flex-row justify-center border border-dashed my-1 hover:z-10" onClick={handleFolderSelection}>
    <VsAdd size={18}class="mx-2 my-1"/> <p> New Folder</p>
  </button>;
};

function initTE(arg0: { Dropdown: any; Ripple: any; }) {
  throw new Error("Function not implemented.");
}

