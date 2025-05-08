<dialog
  data-dialog
  class="rounded-container bg-surface-100-900 text-inherit max-w-[640px] top-1/2 left-1/2 -translate-1/2 p-4 space-y-4 z-10 backdrop:bg-surface-50/75 dark:backdrop:bg-surface-950/75"
>
  <h2 class="h3">Do you wanna play online</h2>
  <p>To configure the lobby or matchbox server used for online go to your settings</p>
  <form method="dialog" class="flex justify-end gap-4">
    <button type="button" class="btn preset-tonal-primary" data-dialog-yes>Yes</button>
    <button type="button" class="btn preset-outlined-tertiary-500" data-dialog-no>No</button>
  </form>
</dialog>


<script lang="ts">
    import { onMount } from 'svelte';
	  import { settingsStore } from '../settings/settingsStore';


    let src = "";

    onMount(() => {
        const urlParams = new URLSearchParams(window.location.search);
        const id = urlParams.get('id');
        const online = urlParams.get("online");


        if (online == "true") {
            const unsubscribe = settingsStore.subscribe(settings => {
                const setSrc = (online: boolean) => src = `/loader.html?name=${id}&lobby=${settings.lobbyName}` + (online ? `&matchbox=${settings.matchboxServer}&lobby_size=${settings.playerCount}` : "" );
                const elemModal: HTMLDialogElement | null = document.querySelector('[data-dialog]');
                const elemTrigger: HTMLButtonElement | null = document.querySelector('[data-dialog-yes]');
                const elemClose: HTMLButtonElement | null = document.querySelector('[data-dialog-no]');

                elemModal?.showModal();

                elemTrigger?.addEventListener('click', () =>{ elemModal?.close(); setSrc(true) });
                elemClose?.addEventListener('click', () => { elemModal?.close(); setSrc(false) });
            });

            return () => unsubscribe();
        } else {
          src = `/loader.html?name=${id}&lobby=test`;
        }
    });
</script>


<iframe id="app-frame" title="game iframe" src={src} style="width: 100%; border: none; height: 100vh"></iframe>