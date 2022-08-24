
const music = new Audio('https://klohger.github.io/resource/media/JAVA_01.mp3');
const wow = document.getElementById('wow');
wow.style.color = "red";
music.loop = true;
let musicEnabled = false;
let musicCanBeEnabled = false;

music.addEventListener("canplaythrough", (event) => {
  /* the audio is now playable; play it if permissions allow */
  musicCanBeEnabled = true;
  
});

function doTheFunny() {
	
	if (musicCanBeEnabled) {
		if(music.paused) {
			console.log('unpausing music');
			music.play();
			wow.style.color = 'green';
			
		} else {
			console.log('pausing music');
			music.pause();
			wow.style.color = 'red';
		}
	}
}