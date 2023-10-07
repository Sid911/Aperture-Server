import { createSignal, onMount } from "solid-js";
import "./App.css";
import { FolderSelector } from "./components/FolderSelector/FolderSelector";
import { Tasks } from "./components/Tasks/Tasks";
import { FiServer } from 'solid-icons/fi'
import Devices from "./components/Devices/Devices";
import {
  Dropdown,
  Ripple,
  initTE,
} from "tw-elements";

import { invoke } from "@tauri-apps/api/tauri";

function App() {
  const [ipAddr, setIpAddr] = createSignal("Loading");
  const [name, setName] = createSignal("");

  onMount(async () => {
    initTE({ Dropdown, Ripple });
    invoke("get_ipv4").then((val) => {
      setIpAddr(val as string)
      console.log(val)
    }).catch((e)=> {
      console.error(e)
      setIpAddr("Unknown :( ")
    })
  })

  return (
    <div class="flex p-5 flex-col">
      <div class="flex flex-row justify-between mb-3">
        <div class="flex flex-row items-center">
          <p class="font-medium text-2xl">
            Aperture <span class="dark:text-neutral-400 font-bold text-lg"> Server </span>
          </p>
          <span><FiServer class="m-2" size={20} /></span>
        </div>
        <button class="dark:bg-black rounded-lg p-2 dark:text-neutral-100 shadow-md shadow-black"> Online : {ipAddr()}</button>
      </div>
      <div class="grid grid-cols-3 gap-5">
        <FolderSelector />
        <Tasks class="col-span-2" tasks={[]} />
      </div>
      <div class="my-4">
        <Devices devices={[
          {
            device_name: "samsung m21",
            device_type: "mobile",
            last_sync_time: "66341723",
            last_update_time: "66556223",
            local_IP: "170.57.210.175",
            os: "Android 13",
            syncing: true
          },
          {
            device_name: "random",
            device_type: "mobile",
            last_sync_time: "403463972377",
            last_update_time: "738463465243",
            local_IP: "141.72.190.43",
            os: "Android 11",
            syncing: false

          },
        ]} ip={ipAddr} />
      </div>
    </div>
  );
}

export default App;
