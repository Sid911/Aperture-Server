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

function App() {
  const [greetMsg, setGreetMsg] = createSignal("");
  const [name, setName] = createSignal("");
  onMount(()=>{initTE({ Dropdown, Ripple });})
  return (
    <div class="flex p-5 flex-col">
      <div class="flex flex-row justify-between mb-3">
        <div class="flex flex-row items-center">
          <p class="font-medium text-2xl">
            Aperture <span class="dark:text-neutral-400 font-bold text-lg"> Server </span>
          </p>
          <span><FiServer class="m-2" size={20} /></span>
        </div>
        <button class="dark:bg-black rounded-lg p-2 dark:text-neutral-100 shadow-md shadow-black"> Online : 192.168.1.9</button>
      </div>
      <div class="grid grid-cols-3 gap-5">
        <FolderSelector />
        <Tasks class="col-span-2" tasks={[
          {
            title: "VMksuRE.pdf",
            description: "From Did",
            file_type: "pdf",
            progress: 0.4,
            total_size: 200000,
            uid: "e13b618e-0111-5077-9de9-d15beebd4c5c"
          }
        ]} />
      </div>
      <div class="my-4">
        <Devices devices={[
          {
            device_name: "did",
            device_type: "mobile",
            last_sync_time: "66341723",
            last_update_time: "66556223",
            local_IP: "170.57.210.175",
            os: "Android 13",
            syncing: true
          },
          {
            device_name: "party",
            device_type: "mobile",
            last_sync_time: "403463972377",
            last_update_time: "738463465243",
            local_IP: "141.72.190.43",
            os: "Android 11",
            syncing: false

          },
          {
            device_name: "eight",
            device_type: "mobile",
            last_sync_time: "394746691527",
            last_update_time: "707815598257",
            local_IP: "182.42.130.81",
            os: "Android 12",
            syncing: false

          },
        ]} />
      </div>
    </div>
  );
}

export default App;
