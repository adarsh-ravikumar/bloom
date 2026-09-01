<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";

  let clientVersion: number[] = $state([0, 0, 0]);
  let clientID: string = $state("");
  let appName: string = $state("");

  let clientInfoRecieved: bool = $state(false);

  let count: number = $state(0);

  onMount(async () => {
    console.log("Mount");

    const unlistenConnected = await listen("client_connected", (event) => {
      console.log("receieved client connection packet");
      console.log(event);
      clientInfoRecieved = true;
      clientVersion = event.payload.client_ver;
      clientID = event.payload.client_id;
      appName = event.payload.app_name;
    });

    const unlistenInc = await listen("increment", (event) => {
      console.log("increment!");
      count++;
    });

    await invoke("frontend_connected");

    return () => {
      unlistenConnected();
      unlistenInc();
    };
  });

  onDestroy(async () => {
    await invoke("frontend_disconnected");
  });

  const handleClick = async () => {
    await invoke("emit_event", {
      event: {
        event: "button_pressed",
        payload: {},
      },
    });
  };
</script>

<h1>Hello, World!</h1>
{#if clientInfoRecieved}
  <div>
    <p>App Name: {appName}</p>
    <p>Client Version: v{clientVersion.join(".")}</p>
    <p>Client ID: {clientID}</p>
  </div>
{/if}

<button onclick={async () => await handleClick()}>Count: {count}</button>

<style>
  button {
    background: black;
    padding: 0.5rem;
    font-size: 1rem;
    color: white;
    border-radius: 0.3rem;
    border: none;
    cursor: pointer;

    transition: 200ms ease;
    scale: 100%;

    &:hover {
      scale: 105%;
    }

    &:active {
      scale: 95%;
    }
  }
</style>
