
const music = new Audio('https://klohger.github.io/media/JAVA_01.ogg');
music.loop = true;

let musicEnabled = false;

let musicCanBeEnabled = false;

const musicButton = document.getElementById('music-button');
const mainTitleStyle = document.getElementById('main-title').style;

music.addEventListener("canplaythrough", (event) => {
  /* the audio is now playable; play it if permissions allow */
  musicCanBeEnabled = true;
  
});

function doTheFunny() {
	
	if (musicCanBeEnabled) {
		if(music.paused) {
			
			music.play();
			musicButton.style.color = 'green';
			mainTitleStyle.animationPlayState = 'running';
			
		} else {
			music.pause();
			musicButton.style.color = 'red';
			mainTitleStyle.animationPlayState = 'paused';
		}
	}
}

musicButton.onclick = doTheFunny;