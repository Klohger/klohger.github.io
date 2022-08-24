const chord = new Audio('https://klohger.github.io/resource/media/JAVA_01.mp3');
const wow = document.getElementById('wow');
chord.loop = true;
myAudioElement.addEventListener("canplaythrough", (event) => {
  /* the audio is now playable; play it if permissions allow */
  myAudioElement.play();
});