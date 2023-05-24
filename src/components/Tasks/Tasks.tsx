import { VsKebabVertical } from "solid-icons/vs";
import { Component, For, JSX, Show } from "solid-js";
import { TaskDropdown } from "./TaskDropdown";

interface TaskProps extends JSX.HTMLAttributes<string>{
  title: string,
  description: string,
  file_type: string,
  uid: string,
  progress: number,
  total_size: number,
}


interface TasksProps extends JSX.HTMLAttributes<string> {
  tasks: TaskProps[]
}

const NoTasks: Component<{}> = (props) => {
  
  return <div class="flex flex-grow justify-center items-center">
    <h3>No Tasks queued  right now</h3>
  </div>;
};

export const Tasks: Component<TasksProps> = (props) => {
  // Probably need to manage the entire thing about rust and db to js here
  // Maybe seperate it into its own class but seems like it will not be as complex
  return (
    <div class={"dark:bg-neutral-800 p-3 dark:text-neutral-100 rounded-lg " + props.class}>
      <span>Tasks</span>
      <Show when={props.tasks.length} fallback={<NoTasks/>}>
      <For each={props.tasks}>{
        (obj, i) => <Task {...obj}/>
      }
      </For>
      </Show>
    </div>);
};

const Task: Component<TaskProps> = (props) => {
  // Probably will need to use Acessors in solid but will figure that out later
  return <div class="dark:bg-neutral-900 p-2 dark:text-neutral-100 round-lg shadow-md shadow-black flex flex-row justify-between items-center my-2">
    <div class="">
      <p class="text-base">{props.title}</p>
      <p class="text-xs text-neutral-300">{props.description}</p>
    </div>
    <div class="h-1 basis-2/3 bg-neutral-200 dark:bg-neutral-600">
      <div class="h-1 bg-cyan-400" style={{width: (props.progress * 100).toString() + "%"}}></div>
    </div>
    <TaskDropdown/>
  </div>;
};