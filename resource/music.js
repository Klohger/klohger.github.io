{
	const music = new Audio('https://klohger.github.io/media/JAVA_01.ogg');
	music.loop = true;

	let musicEnabled = false;

	let musicPlayable = false;

	const musicButton = document.getElementById('music-button');
	const mainTitleStyle = document.getElementById('main-title').style;

	music.addEventListener("canplaythrough", (event) => {
	  /* the audio is now playable; play it if permissions allow */
	  musicPlayable = true;
	});

	music.addEventListener("play", (event) => {
		musicButton.style.color = 'green';
		musicButton.style.outlineColor = 'green';
		mainTitleStyle.animationPlayState = 'running';
	});
	music.addEventListener("pause", (event) => {
		musicButton.style.color = 'red';
		musicButton.style.outlineColor = 'red';
		mainTitleStyle.animationPlayState = 'paused';
	});

	function doTheFunny() {
		
		if (musicPlayable) {
			if(music.paused) {
				music.play();
			} else {
				music.pause();
			}
		}
	}

	musicButton.onclick = doTheFunny;
}