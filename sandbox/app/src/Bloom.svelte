<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let clientVersion: number[] = $state([0, 0, 0]);
  let clientID: string = $state("");
  let appName: string = $state("");

  let clientInfoRecieved: bool = $state(false);

  onMount(async () => {
    await listen("client-connected", (event) => {
      console.log("receieved client connection packet");
      console.log(event);
      clientInfoRecieved = true;
      clientVersion = event.payload.client_ver;
      clientID = event.payload.client_id;
      appName = event.payload.app_name;
    });

    await invoke("emit_event", {
      event: {
        event: "connected",
        payload: {},
      },
    });
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

<button onclick={async () => await handleClick()}>Click!</button>
