<script lang="ts">
	import DevProgress from '$lib/game/DevProgress.svelte';
	import SvgIcon from '$lib/SVGIcon.svelte';
	import Title from '$lib/Title.svelte';
</script>

<div class="flex flex-col h-full w-full">
	<header>
		<Title />
	</header>
	<main class="flex flex-col md:flex-row gap-6 p-4 flex-grow overflow-auto">
		<article class="flex-1 bg-surface-100-800-token rounded-lg p-6">
			<h2 class="text-2xl font-bold mb-4">Welcome !</h2>
			<p class="mb-4">
				This is an highly customazible engine/game for p2p multiplayer 2D top down rogue like.
			</p>

			<div class="mb-6">
				<h3 class="text-xl font-semibold mb-2">Engine: alacod</h3>
				<ul class="list-disc pl-6">
					<li class="mb-1">Random map generator based on LDTK level</li>
					<li class="mb-1">Multiple extendable algorithm for map generation</li>
					<li class="mb-1">P2P networking for serverless game</li>
					<li class="mb-1">Lobby/Matchmaking server that can be self hosted</li>
					<li class="mb-1">Customizable character, weapons, enemies and bonus</li>
				</ul>
			</div>

			<div class="mb-6">
				<h3 class="text-xl font-semibold mb-2">Game: Zombie Alacod</h3>
				<p class="mb-4">
					The engine is develop alongside a clone of call of duty zombie
					gameplay with random map.
				</p>	
			</div>

			<div>
				<h3 class="text-xl font-semibold mb-2">Technologies</h3>
				<ul class="list-disc pl-6">
					<li class="mb-1">Engine: Rust / Bevy</li>
					<li class="mb-1">Networking: Matchbox and GGRS</li>
					<li class="mb-1">Targets: Web , Native and Mobile</li>
				</ul>
			</div>
		</article>

		<aside class="md:w-1/3 lg:w-2/5 flex flex-col gap-6">
      		<DevProgress />
		</aside>
	</main>
	<footer class="mt-auto p-8">
		<div class="rounded-container grid w-full grid-cols-1 gap-4 overflow-hidden md:grid-cols-3">
			<a
				target="_blank"
				class="card flex justify-center preset-filled-surface-100-900 p-4 py-8 text-center hover:scale-105 transition-transform"
				href="https://www.youtube.com/@BasCanada"
			>
				<SvgIcon color="var(--color-primary-500)" id="youtube" />
			</a>
			<a
				target="_blank"
				class="card flex justify-center preset-filled-surface-100-900 p-4 py-8 text-center hover:scale-105 transition-transform"
				href="https://github.com/bascanada/zombie-alacod"
			>
				<SvgIcon color="var(--color-primary-500)" id="github" />
			</a>
			<a
				target="_blank"
				class="card flex justify-center preset-filled-surface-100-900 p-4 py-8 text-center hover:scale-105 transition-transform"
				href="mailto:bascanada1841@gmail.com"
			>
				<SvgIcon color="var(--color-primary-500)" id="gmail" />
			</a>
		</div>
	</footer>
</div>

<style>
	:global(html, body) {
		height: 100%;
	}
	:global(#app) {
		height: 100%;
		display: flex;
		flex-direction: column;
	}
</style>
