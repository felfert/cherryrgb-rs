# This is a writeup of how I attempt to find out the protocol for mapping keys:

## Test environment for packet sniffing

I use a virtualized Windows 10 (with Cherry's utility installed). For virtualization, I use libvirt and virt-manager running on Fedora (36).
After starting the VM, I enable USB Redirection of the keyboard in virt-manager using the Menu: "Virtual Machine" -> "Redirect USB device".
This presents a dialog which allows selecting the device(s) to be redirected to the windows guest. After enabling redirection, in the
windows guest, the GUI of the Cherry utility should appear. The keyboard is no longer available in the Linux host. Therfore you should
have a second (different) keyboard connected to the host.

Now, start tshark using the following script:
```bash
#!/bin/sh
line="$(lsusb -d 046a:00df)"
bus=$(echo "${line}" | awk '{print $2}')
dev=$(echo "${line}" | awk '{print $4}' | tr -d :)
tshark -i usbmon0 -e usb.data_fragment -T fields -l -Y "usb.bus_id == ${bus} and usb.device_address == ${dev} and usb.src == host and usb.data_fragment > 0"
```
The 046a:00df above is the VendorID:ProductID of my MX 10.0 keyboard.

After that, in the guest, start mapping a key and observe the output of tshark in the terminal window after hitting "Apply" in the GUI.
It should print 12 lines of hex values. Each line represents a control packets sent to the device.
The first three packets are device info requests. The 4th ist a "Start transaction". Following are 7 frames representing the actual
mapping. Finally the 12th packet represents an "End transaction"

## Example dump of Mapping key "A" to "B"
<pre>
04250003220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // Fetch device info
04250003220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // Fetch device info
04250003220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // Fetch device info
04010001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // Start transaction
044405093800000020002920003520002b20003920020020010020004820001e200014<b>2000<i>05</i></b>200064200800a0030020001f20001a20001620001d2004002000
043d0509383800003a20002020000820000720001b20000020003b20002120001520000920000620000020003c20002220001720000a20001920008b20003d20
0402060938700000002320001c20000b20000520002c20003e20002420001820000d20001120800020003f20002520000c20000e20001020008a200040200026
0449070938a8000020001220000f20003620008820004120002720001320003320003720400020004220002d20002f200034200038a0010020004320002e2000
0456080938e000003020003220008720006520004420002a20003120002820200020100020004520004920004c20008920000020005020004620004a20004d20
04d0080938180100000020005220005120004720004b20004e20000020000020004f30ea0020005320005f20005c20005920000030e20020005420006020005d
047b0a092a50010020005a20006230e90020005520006120005e20005b200063309201200056200057200085200058200000000030e20020005420006020005d
04020002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // End transaction
</pre>
As you can see above, the byte at (zero-based) index 38 of the first mapping frame is set to 5. Actually, the USB HID spec
defines 16 bit scan codes. The code 0x0005 is assigned to the letter "B".
## Example dump of deleting the previous mapping, restoring the keyboard to the "No mapping at all" state.
<pre>
04250003220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // Fetch device info
04250003220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // Fetch device info
04250003220000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // Fetch device info
04010001000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // Start transaction
044405093800000020002920003520002b20003920020020010020004820001e200014<b>2000<i>04</i></b>200064200800a0030020001f20001a20001620001d2004002000
043d0509383800003a20002020000820000720001b20000020003b20002120001520000920000620000020003c20002220001720000a20001920008b20003d20
0402060938700000002320001c20000b20000520002c20003e20002420001820000d20001120800020003f20002520000c20000e20001020008a200040200026
0449070938a8000020001220000f20003620008820004120002720001320003320003720400020004220002d20002f200034200038a0010020004320002e2000
0456080938e000003020003220008720006520004420002a20003120002820200020100020004520004920004c20008920000020005020004620004a20004d20
04d0080938180100000020005220005120004720004b20004e20000020000020004f30ea0020005320005f20005c20005920000030e20020005420006020005d
047b0a092a50010020005a20006230e90020005520006120005e20005b200063309201200056200057200085200058200000000030e20020005420006020005d
04020002000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000 // End transaction
</pre>
This time, the same byte ist set back to 4. (The scan code 0x0004 is assigned to the letter "A")
## Table of USB HID scan codes
A table of USB keyboard scan codes can be found in the [USB HID specification](https://usb.org/sites/default/files/hut1_21.pdf) at chapter
10. Keyboard/Keypad Page (0x07)s starting at page 82. (Named Usage ID in that document). It is a 16bit value with the MSB usually being 0x00 for
most "regular" keys.
## Keyboard mapping frames

It appears, that for each key, 3 consecutive bytes are used at a specific index of one of the 7 mapping frames (following the start transaction).
Each mapping frame starts with the following sequence:

<pre>
     Checksum
     │
     │   Operation 9 == Map keys?
     │    │
     │    │   Some index?
   ┌─┴─┐  │ ┌─┴─┐
04 CS CS 09 ?? ??
</pre>
The offsets for the individual keys are shown in the follwing annotated dump. The key names are mostly taken verbatim from the labels of my MX 10.0
(german layout). The first two byte of the "normal" keys usually are 0x20 0x00. The 3rd byte seem to be the scan code (Usage ID). For the modifier keys,
the second byte has a different value (perhaps a bitmap of the modifier types?). Another exception are the 4 keys in the uppermost row above the numeric
block (from left to right: Vol-, Mute, Vol+ and Calc). Those have a completely different byte sequence, starting with 0x30. Finally, there is the
"CHERRY" key, which does not even exist on my MX 10.0 but regardless is configurable in the Windows cherry utility. Also, 4 keys have duplicate entries
where BOTH of the entries (in the 6th and 7th frame) are set by the Windows cherry utility: Mute, Num/, Num8 and Num5. Some unlabeled byte sequences
appear to be associated to keys that are not available in the german layout of my keyboard.
<pre>
                 ESC    °(`)   Tab    CapsLk LShift LCtrl  Pause  1      Q      A      <(k45) LWin   CHERRY 2      W      S      Y(Z)   LAlt
0443050938000000 200029 200035 20002b 200039 200200 200100 200048 20001e 200014 200004 200064 200800 a00300 20001f 20001a 200016 20001d 200400 2000

             F1     3      E      D      X             F2     4      R      F      C             F3     5      T      G      V             F4
043d05093838 00003a 200020 200008 200007 20001b 200000 20003b 200021 200015 200009 200006 200000 20003c 200022 200017 20000a 200019 20008b 20003d 20

               6      Z(Y)   H      B      Space  F5     7      U      J      N             F6     8      I      K      M             F7     9
04020609387000 000023 20001c 20000b 200005 20002c 20003e 200024 200018 20000d 200011 208000 20003f 200025 20000c 20000e 200010 20008a 200040 200026

                 O      L      ,             F8     0      P      Ö(;)   .      RAlt   F9     ß(-)   Ü([)   Ä(')   -(/)          F10    ´(=)
0449070938a80000 200012 20000f 200036 200088 200041 200027 200013 200033 200037 204000 200042 20002d 20002f 200034 200038 a00100 200043 20002e 2000

             +(])   #(k42)        App    F11    BS            Enter  RShift RCtrl  F12    Ins    Del                 Left   PrtScr Pos1   End
0456080938e0 000030 200032 200087 200065 200044 20002a 200031 200028 202000 201000 200045 200049 20004c 200089200000 200050 200046 20004a 20004d 20

                     Up     Down   ScrLck PgUp   PgDn                Right  Vol-   NumLk  Num7   Num4   Num1          Mute   Num/   Num8   Num5
04d00809381801000000 200052 200051 200047 20004b 20004e 200000200000 20004f 30ea00 200053 20005f 20005c 200059 200000 30e200 200054 200060 20005d

                 Num2   Num0   Vol+   Num*   Num9   Num6   Num3   Num,   Calc   Num-   Num+          NumEnt            Mute   Num/   Num8   Num5 
047b0a092a500100 20005a 200062 30e900 200055 200061 20005e 20005b 200063 309201 200056 200057 200085 200058 2000000000 30e200 200054 200060 20005d
</pre>
