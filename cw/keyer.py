import argparse
import numpy as np
import pathlib
import sounddevice as sd
import threading
import queue
import sys

from evdev import InputDevice, ecodes

# globals 

LEFT_PRESSED   = "LEFT_PRESSED"
LEFT_RELEASED  = "LEFT_RELEASED"
RIGHT_PRESSED  = "RIGHT_PRESSED"
RIGHT_RELEASED = "RIGHT_RELEASED"

# snd: audio control part

def tone(freq, duration, samplerate=44100, volume=0.5):
    t = np.linspace(0, duration, int(samplerate * duration), endpoint=False)
    return volume * np.sin(2 * np.pi * freq * t)


def silence(duration, samplerate=44100):
    return np.zeros(int(samplerate * duration))


def generate_samples(cw):
    # Morse timing: PARIS = 50 units
    unit = 60.0 / (cw.wpm * 50)

    audio = []
    audio.append(silence(1 * unit, cw.samplerate))
    isi = np.concatenate(audio)

    audio = []
    audio.append(tone(cw.freq, 1 * unit, cw.samplerate, cw.volume))
    audio.append(silence(1 * unit, cw.samplerate))
    dit = np.concatenate(audio)

    audio = []
    audio.append(tone(cw.freq, 3 * unit, cw.samplerate, cw.volume))
    audio.append(silence(1 * unit, cw.samplerate))
    dah = np.concatenate(audio)

    return isi, dit, dah


def play_sample(wave, samplerate=44100):
    sd.play(wave, samplerate)
    sd.wait()

# Note: currenty only Iambic-A type is supported
def audio_thread(ch: queue.Queue, cw: dict):

    isi, dit, dah = generate_samples(cw)

    next_element = "dit"  # alternation seed
    right_down = False
    left_down = False
    exit = False

    while not exit:
        while True:
            try:
                ev = ch.get_nowait()
                ch.task_done()
            except queue.Empty:
                break

            if ev is None:
                exit = True
                break

            if ev == LEFT_PRESSED:
                left_down = True
            elif ev == LEFT_RELEASED:
                left_down = False
            elif ev == RIGHT_PRESSED:
                right_down = True
            elif ev == RIGHT_RELEASED:
                right_down = False

        if left_down and not right_down:
            play_sample(dit, samplerate=cw.samplerate)

        elif right_down and not left_down:
            play_sample(dah, samplerate=cw.samplerate)

        elif left_down and right_down:
            if next_element == "dit":
                play_sample(dit, samplerate=cw.samplerate)
                next_element = "dah"
            else:
                play_sample(dah, samplerate=cw.samplerate)
                next_element = "dit"

        else:
            # idle: nothing pressed
            next_element = "dit"  # reset alternation
            #play_sample(isi, samplerate=cw.samplerate)


def create_parser():
    """ Parse command line arguments """
    parser = argparse.ArgumentParser()

    parser.add_argument('-d', '--device', action='store', type=str,
                        required=True, dest='device', help='event /dev/input/eventX file')
    parser.add_argument('-v', '--volume', action='store', type=int, default=1,
                        required=False, dest='volume', help='cw volume')
    parser.add_argument('-w', '--wpm', action='store', type=int, default=20,
                        required=False, dest='wpm', help='wpm speed (words per minute)')
    parser.add_argument('-f', '--freq', action='store', type=int, default=600,
                        required=False, dest='freq', help='cw tone frequency')
    parser.add_argument('-r', '--rate', action='store', type=int, default=44100,
                        required=False, dest='samplerate', help='audio sample rate')
    parser.add_argument('-x', '--verbose', action='store_true', required=False,
                              dest='verbose', help='verbose mode')
    return parser


if __name__ == '__main__':
    cmdline = create_parser()
    cw = cmdline.parse_args()

    if not pathlib.Path(cw.device).exists():
        print(f"Event file does not exists: {cw.device}")
        sys.exit(-1)

    dev = InputDevice(cw.device)
    dev.grab()

    print(f"CW properties: {cw}")
    print(dev)

    chan = queue.Queue()
    t = threading.Thread(target=audio_thread, args=(chan, cw))
    t.start()

    right_rattle = 0
    left_rattle = 0

    for event in dev.read_loop():
        if event.type == ecodes.EV_KEY:
            state = event.value
            key = event.code

            if key == ecodes.BTN_LEFT and state == 1:
                chan.put(LEFT_PRESSED)
                left_rattle += 1
                right_rattle = 0

            if key == ecodes.BTN_LEFT and state == 0:
                chan.put(LEFT_RELEASED)

            if key == ecodes.BTN_RIGHT and state == 1:
                chan.put(RIGHT_PRESSED)
                right_rattle += 1
                left_rattle = 0

            if key == ecodes.BTN_RIGHT and state == 0:
                chan.put(RIGHT_RELEASED)

        # press button on adapter board to exit
        if event.type == ecodes.EV_REL:
            key = event.code
            if key == 1:
                break

        # alternatively press left or right button 5 times
        if left_rattle >= 5 or right_rattle >= 5:
            break

    chan.put(None)   # close channel
    t.join()

    dev.ungrab()
    sys.exit(0)
