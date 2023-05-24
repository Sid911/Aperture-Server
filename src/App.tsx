import { createSignal } from "solid-js";
import "./App.css";
import { FolderSelector } from "./components/FolderSelector/FolderSelector";
import { Tasks } from "./components/Tasks/Tasks";
import { FiServer } from 'solid-icons/fi'
import Devices from "./components/Devices/Devices";

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");

  return (
    <div class="flex p-5 flex-col">
      <div class="flex flex-row justify-between mb-3">
        <div class="flex flex-row items-center">
        <p class="font-medium text-2xl">
          Aperture <span class="dark:text-neutral-400 font-bold text-lg"> Server </span>
        </p>
        <span><FiServer class="m-2" size={20}/></span>
        </div>
        <button class="dark:bg-black rounded-lg p-2 dark:text-neutral-100 shadow-md shadow-black"> Online</button>
      </div>
      <div class="grid grid-cols-3 gap-5">
      <FolderSelector/>
      <Tasks class="col-span-2" tasks={[]}/>
      </div>
      <div class="my-4">
        <Devices devices={[]}/>
      </div>
    </div>
  );
}

export default App;
