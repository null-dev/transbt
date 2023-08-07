# transbt

Copy paired Bluetooth devices from a Linux system to one or more Windows systems. Useful when dual-booting between Windows and Linux.

## Usage 
1. Boot into Windows.
2. Pair the Bluetooth device(s) to the Windows system.
3. Reboot into Linux.
4. Pair the Bluetooth device(s) to the Linux system, this will overwrite the pairing with the Windows system on the Bluetooth device.
5. Run `sudo transbt dump`. This will dump all paired Bluetooth devices from the Linux system into a file called `dump.json`.
6. Copy `dump.json` onto your Windows partition.
7. Reboot into Windows.
8. Execute `transbt list` to show all the devices in the Bluetooth dump. The devices will be grouped by Bluetooth adapter.
9. Execute `transbt apply <ADAPTER MAC ADDRESS> <DEVICE MAC ADDRESS>` to copy a Bluetooth device from the dump to the Windows system.
   
   - **This command must be run as the `SYSTEM` user**, see `run.ps1` for an example on how to do this.
   - Example command line: `transbt apply aa:bb:cc:dd:ee:ff zz:yy:xx:ww:vv:uu`
10. Reboot Windows, and with any luck, your Bluetooth devices will now be working!