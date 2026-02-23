import argparse
import numpy as np
import sounddevice as sd
import sys

MORSE = {
    # Letters
    'A': '.-',    'B': '-...',  'C': '-.-.',  'D': '-..',
    'E': '.',     'F': '..-.',  'G': '--.',   'H': '....',
    'I': '..',    'J': '.---',  'K': '-.-',   'L': '.-..',
    'M': '--',    'N': '-.',    'O': '---',   'P': '.--.',
    'Q': '--.-',  'R': '.-.',   'S': '...',   'T': '-',
    'U': '..-',   'V': '...-',  'W': '.--',   'X': '-..-',
    'Y': '-.--',  'Z': '--..',

    # Digits
    '0': '-----', '1': '.----', '2': '..---', '3': '...--',
    '4': '....-', '5': '.....', '6': '-....', '7': '--...',
    '8': '---..', '9': '----.',

    # Punctuation
    '.': '.-.-.-',    # period
    ',': '--..--',    # comma
    '?': '..--..',    # question mark
    "'": '.----.',    # apostrophe
    '!': '-.-.--',    # exclamation mark
    '/': '-..-.',     # slash
    '(': '-.--.',     # open parenthesis
    ')': '-.--.-',    # close parenthesis
    '&': '.-...',     # ampersand
    ':': '---...',    # colon
    ';': '-.-.-.',    # semicolon
    '=': '-...-',     # equals
    '+': '.-.-.',     # plus
    '-': '-....-',    # hyphen
    '_': '..--.-',    # underscore
    '"': '.-..-.',    # quotation mark
    '$': '...-..-',   # dollar sign
    '@': '.--.-.',    # at sign
}


def tone(freq, duration, samplerate=44100, volume=0.5):
    t = np.linspace(0, duration, int(samplerate * duration), endpoint=False)
    return volume * np.sin(2 * np.pi * freq * t)

def silence(duration, samplerate=44100):
    return np.zeros(int(samplerate * duration))

def play_morse(text, cw):
    # Morse timing: PARIS = 50 units
    unit = 60.0 / (cw.wpm * 50)
    audio = []

    # warmup silence
    audio.append(silence(0.1, cw.samplerate))  # 100 ms

    for i, char in enumerate(text.upper()):
        if char == ' ':
            audio.append(silence(4 * unit, cw.samplerate)) # required 7 = 4 (this) + 3 (see inter-symbol delay below)
            continue

        code = MORSE.get(char)
        if not code:
            continue

        for j, symbol in enumerate(code):
            if symbol == '.':
                audio.append(tone(cw.freq, 1 * unit, cw.samplerate, cw.volume))
            elif symbol == '-':
                audio.append(tone(cw.freq, 3 * unit, cw.samplerate, cw.volume))

            if j < len(code) - 1:
                audio.append(silence(1 * unit, cw.samplerate))

        if i < len(text) - 1:
            audio.append(silence(3 * unit, cw.samplerate))

    # shutdown silence
    audio.append(silence(0.5, cw.samplerate))  # 500 ms

    wave = np.concatenate(audio)
    sd.play(wave, cw.samplerate)
    sd.wait()

def create_parser():
    """ Parse command line arguments """
    parser = argparse.ArgumentParser()

    parser.add_argument('-v', '--volume', action='store', type=int, default=1,
                        required=False, dest='volume', help='cw volume')
    parser.add_argument('-w', '--wpm', action='store', type=int, default=20,
                        required=False, dest='wpm', help='wpm speed (words per minute)')
    parser.add_argument('-f', '--freq', action='store', type=int, default=600,
                        required=False, dest='freq', help='cw tone frequency')
    parser.add_argument('-r', '--rate', action='store', type=int, default=44100,
                        required=False, dest='samplerate', help='audio sample rate')

    parser.add_argument(
        "file",
        nargs="?",
        type=argparse.FileType("r"),
        default=sys.stdin,
        help="Input file (default: stdin)",
    )

    return parser

if __name__ == '__main__':
    cmdline = create_parser()
    cw = cmdline.parse_args()

    for line in cw.file:
        line.rstrip('\n')
        play_morse(line, cw)
