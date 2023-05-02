# MIDI

If you're running Windows, MIDI ecalls should work out of the box! On Linux, however, you need
to install a software synthesizer and a soundfont. I recommend following [the Timidity++ guide](https://wiki.archlinux.org/title/Timidity++)
until the section that explains how to run the daemon.

Note that `systemctl start timidity` may fail with the error "Failed to start
timidity.service: Unit timidity.service not found.", in which case you may try
`systemctl --user start timidity` instead, or run `timidity -iA` <strike>as a last
resort</strike>.

FPGRARS takes a `--port` argument that specifies the MIDI port to use. You can
find the port number by running `aplaymidi -l` and finding a TiMidity port. Although in my 
experience some of them might not work :)


