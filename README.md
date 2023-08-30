# Tangible Soundscapes - Sonic Thinking 2023

This project was created as part of the *Sonic Thinking* and *Design Thinking for Digital Engineering* courses
in the summer term 2023 at HPI.

It presents a proof of concept for using miniatures equipped with RFID
chips to dynamically generate ambient soundscapes in tabletop RPGs.

There is also a [demo video](https://files.tiedt.dev/demo.mp4).

## Instructions

The RFID readers are MFRC522 modules connected to an STM32F401RE board. Build the image for the board by calling
`make` in the `stm32_figure_reader` directory and flash `node.bin` to the board.

Connect the board to a PC with a Mini USB cable and run the `tangible_soundscape_player` binary with the board's serial
port and the file containing the ruleset as arguments.

## Used sounds

The example sounds distributed here are taken from freesound.org.

- "Ambience, Night Wildlife, A.wav" by InspectorJ (www.jshaw.co.uk)
- "10347 single bell campanile loop.wav" by Robinhood76
- "Owl.WAV" by inchadney
- "Birds in the wood.wav" by emilgasi
- "1203_hungarian_sheep.wav" reinsamba
- "tree_creak_04.wav" by Department64
- "frogs in a pond" by eastierp
- "Stream Running Into Pond" by mhtaylor67