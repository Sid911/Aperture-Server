import { Component, For, Show } from "solid-js";
import { FaSolidMobile, FaSolidPlus } from "solid-icons/fa";
import Smartphone from "../../assets/smartphone2.png";

interface DeviceProps {
  device_name: string;
  local_IP: string;
  last_sync_time: string;
  last_update_time: string;
  device_type: string;
  os: string;
  syncing: boolean;
}

interface DevicesProps {
  devices: DeviceProps[];
}

const NoDevices: Component<{}> = (props) => {
  return (
    <div class="flex justify-center items-center flex-grow">
      <h3>No devices found</h3>
    </div>
  );
};

const Devices: Component<DevicesProps> = (props) => {
  return (
    <div class="dark:bg-neutral-800 bg-neutral-100 rounded-lg flex flex-col border-2 dark:border-black">
      <div class="flex flex-row items-center p-4">
        Devices{" "}
        <span>
          <FaSolidMobile size={18} class="m-2" />
        </span>

        <button class="dark:bg-black p-2 rounded-md mx-2"><FaSolidPlus/></button>
      </div>
      <div class="flex flex-wrap overscroll-y-contain h-96 overflow-scroll">
        <Show when={props.devices.length} fallback={<NoDevices />}>
          <For each={props.devices}>{(obj, i) => <Device {...obj} />}</For>
        </Show>
      </div>
    </div>
  );
};

const Device: Component<DeviceProps> = (props) => {
  const style = "flex flex-row max-w-sm m-3 bg-neutral-900 max-h-72 shadow-xl shadow-cyan-400/10 transition-all hover:shadow-black hover:shadow-2xl rounded-xl hover:-translate-y-2" + (props.syncing? " border-2 border-cyan-400": "");
  return (
    <div class={style}>
      <div>
        <img src={Smartphone} alt="phone" srcset="" class="p-3 max-h-80" />
      </div>
      <div class="flex flex-col justify-between p-6">
        Device:{" "}
        <h5 class="mb-2 text-xl font-thin text-neutral-800 dark:text-neutral-50 first-letter:uppercase first-letter:font-medium">
          {props.device_name}
        </h5>
        <div class="mb-4 text-sm text-neutral-600 dark:text-neutral-200">
          <table>
            <tbody>
              <tr>
                <td>IP</td>
                <td>{props.local_IP}</td>
              </tr>
              <tr>
                <td>Last sync : </td>
                <td> {props.last_sync_time}</td>
              </tr>
            </tbody>
          </table>
        </div>
        <p class="text-xs text-neutral-500 dark:text-neutral-300">
          Last updated {props.last_update_time}
        </p>
      </div>
    </div>
  );
};

export default Devices;
