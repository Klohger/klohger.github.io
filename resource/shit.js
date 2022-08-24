const chord = new Audio('https://klohger.github.io/resource/media/JAVA_01.mp3');
const wow = document.getElementById('wow');
chord.loop = true;


function doTheFunny() {
	wow.style.color = 'red';
	chord.play();
}