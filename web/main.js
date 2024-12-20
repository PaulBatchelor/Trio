import { startAudio, getGlobalTrio } from "./audio.js";
const AudioContext = window.AudioContext || window.webkitAudioContext;
//const audioContext = new AudioContext({sampleRate: 44100});
const audioContext = new AudioContext();

const canvas = document.getElementById('canvas');
const html = document.getElementsByTagName('html')[0];
const container = document.getElementById('container');

canvas.width = 500;
canvas.height = 500;
const ctx = canvas.getContext('2d');

let gate = false
let circX = -1;
let circY = -1;
let totalSteps = 13;
let audioStarted = false;

let trioNode = null;

let portraitMode = false;

function getStepWidth(portraitMode, width, height) {
    if (portraitMode) {
        return canvas.height / totalSteps;
    }

    return canvas.width / totalSteps;
}

function clamp(x, mn, mx) {
    x = x < mn ? mn : x;
    x = x > mx ? mx : x;
    return x;
}

function sendMoveEvent(xpos, ypos) {
    if (trioNode == null || !gate) {
        return;
    }

    if (canvas.width == 0 || canvas.height == 0) {
        return;
    }

    let xpos_norm = clamp(xpos / canvas.width, 0, 1);
    let ypos_norm = clamp(ypos / canvas.height, 0, 1);

    if (portraitMode) {
        trioNode.move(1 - ypos_norm, 1 - xpos_norm);
    } else {
        trioNode.move(xpos_norm, ypos_norm);
    }
}

function sendGateEvent(turnOn) {
    if (trioNode == null) {
        return;
    }

    if (turnOn) {
        trioNode.on();
    } else {
        trioNode.off();
    }
}


function draw() {
    canvas.width = html.clientWidth;
    canvas.height = html.clientHeight;
    ctx.fillStyle = '#006262';
    ctx.fillRect(0, 0, canvas.width, canvas.height);

    if (!audioStarted) {
        ctx.textAlign = 'center';
        ctx.fillStyle = '#FFF';
        ctx.font = "bold 48px sans-serif";
        ctx.fillText("TAP TO BEGIN",
            canvas.width / 2,
            canvas.height / 2);
        let raf = window.requestAnimationFrame(draw);
        return;
    }

    ctx.lineWidth = 3;
    ctx.strokeStyle = '#FFF';


    portraitMode = canvas.height > canvas.width;

    let stepWidth = getStepWidth(portraitMode, canvas.width, canvas.height);

    for (let i = 1; i < totalSteps; i++) {
        if (portraitMode) {
            ctx.beginPath();
            ctx.moveTo(0, i*stepWidth);
            ctx.lineTo(canvas.width, i*stepWidth);
            ctx.closePath();
            ctx.stroke();
        } else {
            ctx.beginPath();
            ctx.moveTo(i*stepWidth, 0);
            ctx.lineTo(i*stepWidth, canvas.height);
            ctx.closePath();
            ctx.stroke();
        }
    }

    ctx.fillStyle = '#FFFFFF';
    if (gate && circX >= 0 && circY >= 0) {
        ctx.beginPath();
        ctx.arc(circX, circY, 50, 0, 2 * Math.PI, true);
        ctx.closePath();
        ctx.fill();
    }


    let raf = window.requestAnimationFrame(draw);
}

var curPointer = -1;
function down(event) {
    //if (curPointer == -1) {
        console.log(event.pointerId);
        circX = event.clientX;
        circY = event.clientY;
        gate = true;
        sendGateEvent(true);
        sendMoveEvent(circX, circY);
        curPointer = event.pointerId;
    //}
}

function up(event) {
    gate = false;
    sendGateEvent(false);

    //if (event.pointerId == curPointer) {
        curPointer = -1;
    //}
}

function move(event) {
    //if (event.pointerId == curPointer) {
        circX = event.clientX;
        circY = event.clientY;
        sendMoveEvent(circX, circY);
    //}
}

// canvas.addEventListener('mousedown', down);
// canvas.addEventListener('mousemove', move);
// canvas.addEventListener('mouseup', up);

container.addEventListener('click', async () => {
    if (!audioStarted) {
        console.log("starting!");
        //startAudio(audioContext).then((result) => {
        //    trioNode = result;
        //});
        await startAudio(audioContext);
        console.log("started!");
        audioStarted = true;
        audioContext.resume();
        trioNode = getGlobalTrio();
    }
});


canvas.addEventListener('pointerdown', down, false);
canvas.addEventListener('pointermove', move, false);
canvas.addEventListener('pointerup', up, false);

draw();
