# This is a writeup of how I attemt to find out the protocol for mapping keys:

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
As you can see above, the byte at (zero-based) index 38 of the first mapping frame is set to 5.
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
This time, the same byte ist set back to 4.
## Table of scan codes
<pre><table>
<tr><th align="right">SC</th><th>Key</th></tr>
<tr><td>01</td><td>VK_ESCAPE</td></tr>
<tr><td>02</td><td>VK_1</td></tr>
<tr><td>03</td><td>VK_2</td></tr>
<tr><td>04</td><td>VK_3</td></tr>
<tr><td>05</td><td>VK_4</td></tr>
<tr><td>06</td><td>VK_5</td></tr>
<tr><td>07</td><td>VK_6</td></tr>
<tr><td>08</td><td>VK_7</td></tr>
<tr><td>09</td><td>VK_8</td></tr>
<tr><td>0A</td><td>VK_9</td></tr>
<tr><td>0B</td><td>VK_0</td></tr>
<tr><td>0C</td><td>VK_OEM_4</td></tr>
<tr><td>0D</td><td>VK_OEM_6</td></tr>
<tr><td>0E</td><td>VK_BACK</td></tr>
<tr><td>0F</td><td>VK_TAB</td></tr>
<tr><td>10</td><td>VK_Q</td></tr>
<tr><td>11</td><td>VK_W</td></tr>
<tr><td>12</td><td>VK_E</td></tr>
<tr><td>13</td><td>VK_R</td></tr>
<tr><td>14</td><td>VK_T</td></tr>
<tr><td>15</td><td>VK_Z</td></tr>
<tr><td>16</td><td>VK_U</td></tr>
<tr><td>17</td><td>VK_I</td></tr>
<tr><td>18</td><td>VK_O</td></tr>
<tr><td>19</td><td>VK_P</td></tr>
<tr><td>1A</td><td>VK_OEM_1</td></tr>
<tr><td>1B</td><td>VK_OEM_PLUS</td></tr>
<tr><td>1C</td><td>VK_RETURN</td></tr>
<tr><td>1D</td><td>VK_LCONTROL</td></tr>
<tr><td>1E</td><td>VK_A</td></tr>
<tr><td>1F</td><td>VK_S</td></tr>
<tr><td>20</td><td>VK_D</td></tr>
<tr><td>21</td><td>VK_F</td></tr>
<tr><td>22</td><td>VK_G</td></tr>
<tr><td>23</td><td>VK_H</td></tr>
<tr><td>24</td><td>VK_J</td></tr>
<tr><td>25</td><td>VK_K</td></tr>
<tr><td>26</td><td>VK_L</td></tr>
<tr><td>27</td><td>VK_OEM_3</td></tr>
<tr><td>28</td><td>VK_OEM_7</td></tr>
<tr><td>29</td><td>VK_OEM_5</td></tr>
<tr><td>2A</td><td>VK_LSHIFT</td></tr>
<tr><td>2B</td><td>VK_OEM_2</td></tr>
<tr><td>2C</td><td>VK_Y</td></tr>
<tr><td>2D</td><td>VK_X</td></tr>
<tr><td>2E</td><td>VK_C</td></tr>
<tr><td>2F</td><td>VK_V</td></tr>
<tr><td>30</td><td>VK_B</td></tr>
<tr><td>31</td><td>VK_N</td></tr>
<tr><td>32</td><td>VK_M</td></tr>
<tr><td>33</td><td>VK_OEM_COMMA</td></tr>
<tr><td>34</td><td>VK_OEM_PERIOD</td></tr>
<tr><td>35</td><td>VK_OEM_MINUS</td></tr>
<tr><td>36</td><td>VK_RSHIFT</td></tr>
<tr><td>37</td><td>VK_MULTIPLY</td></tr>
<tr><td>38</td><td>VK_LMENU</td></tr>
<tr><td>39</td><td>VK_SPACE</td></tr>
<tr><td>3A</td><td>VK_CAPITAL</td></tr>
<tr><td>3B</td><td>VK_F1</td></tr>
<tr><td>3C</td><td>VK_F2</td></tr>
<tr><td>3D</td><td>VK_F3</td></tr>
<tr><td>3E</td><td>VK_F4</td></tr>
<tr><td>3F</td><td>VK_F5</td></tr>
<tr><td>40</td><td>VK_F6</td></tr>
<tr><td>41</td><td>VK_F7</td></tr>
<tr><td>42</td><td>VK_F8</td></tr>
<tr><td>43</td><td>VK_F9</td></tr>
<tr><td>44</td><td>VK_F10</td></tr>
<tr><td>45</td><td>VK_NUMLOCK</td></tr>
<tr><td>46</td><td>VK_SCROLL</td></tr>
<tr><td>47</td><td>VK_HOME</td></tr>
<tr><td>48</td><td>VK_UP</td></tr>
<tr><td>49</td><td>VK_PRIOR</td></tr>
<tr><td>4A</td><td>VK_SUBTRACT</td></tr>
<tr><td>4B</td><td>VK_LEFT</td></tr>
<tr><td>4C</td><td>VK_CLEAR</td></tr>
<tr><td>4D</td><td>VK_RIGHT</td></tr>
<tr><td>4E</td><td>VK_ADD</td></tr>
<tr><td>4F</td><td>VK_END</td></tr>
<tr><td>50</td><td>VK_DOWN</td></tr>
<tr><td>51</td><td>VK_NEXT</td></tr>
<tr><td>52</td><td>VK_INSERT</td></tr>
<tr><td>53</td><td>VK_DELETE</td></tr>
<tr><td>54</td><td>VK_SNAPSHOT</td></tr>
<tr><td>56</td><td>VK_OEM_102</td></tr>
<tr><td>57</td><td>VK_F11</td></tr>
<tr><td>58</td><td>VK_F12</td></tr>
<tr><td>59</td><td>VK_CLEAR</td></tr>
<tr><td>5A</td><td>VK_OEM_WSCTRL</td></tr>
<tr><td>5B</td><td>VK_DBE_KATAKANA</td></tr>
<tr><td>5C</td><td>VK_OEM_JUMP</td></tr>
<tr><td>5D</td><td>VK_DBE_FLUSHSTRING</td></tr>
<tr><td>5E</td><td>VK_OEM_BACKTAB</td></tr>
<tr><td>5F</td><td>VK_OEM_AUTO</td></tr>
<tr><td>62</td><td>VK_DBE_NOCODEINPUT</td></tr>
<tr><td>63</td><td>VK_HELP</td></tr>
<tr><td>64</td><td>VK_F13</td></tr>
<tr><td>65</td><td>VK_F14</td></tr>
<tr><td>66</td><td>VK_F15</td></tr>
<tr><td>67</td><td>VK_F16</td></tr>
<tr><td>68</td><td>VK_F17</td></tr>
<tr><td>69</td><td>VK_F18</td></tr>
<tr><td>6A</td><td>VK_F19</td></tr>
<tr><td>6B</td><td>VK_F20</td></tr>
<tr><td>6C</td><td>VK_F21</td></tr>
<tr><td>6D</td><td>VK_F22</td></tr>
<tr><td>6E</td><td>VK_F23</td></tr>
<tr><td>6F</td><td>VK_OEM_PA3</td></tr>
<tr><td>71</td><td>VK_OEM_RESET</td></tr>
<tr><td>73</td><td>VK_ABNT_C1</td></tr>
<tr><td>76</td><td>VK_F24</td></tr>
<tr><td>7B</td><td>VK_OEM_PA1</td></tr>
<tr><td>7C</td><td>VK_TAB</td></tr>
<tr><td>7E</td><td>VK_ABNT_C2</td></tr>
<tr><td>E0 10</td><td>VK_MEDIA_PREV_TRACK</td></tr>
<tr><td>E0 19</td><td>VK_MEDIA_NEXT_TRACK</td></tr>
<tr><td>E0 1C</td><td>VK_RETURN</td></tr>
<tr><td>E0 1D</td><td>VK_RCONTROL</td></tr>
<tr><td>E0 20</td><td>VK_VOLUME_MUTE</td></tr>
<tr><td>E0 21</td><td>VK_LAUNCH_APP2</td></tr>
<tr><td>E0 22</td><td>VK_MEDIA_PLAY_PAUSE</td></tr>
<tr><td>E0 24</td><td>VK_MEDIA_STOP</td></tr>
<tr><td>E0 2E</td><td>VK_VOLUME_DOWN</td></tr>
<tr><td>E0 30</td><td>VK_VOLUME_UP</td></tr>
<tr><td>E0 32</td><td>VK_BROWSER_HOME</td></tr>
<tr><td>E0 35</td><td>VK_DIVIDE</td></tr>
<tr><td>E0 37</td><td>VK_SNAPSHOT</td></tr>
<tr><td>E0 38</td><td>VK_RMENU</td></tr>
<tr><td>E0 46</td><td>VK_CANCEL</td></tr>
<tr><td>E0 47</td><td>VK_HOME</td></tr>
<tr><td>E0 48</td><td>VK_UP</td></tr>
<tr><td>E0 49</td><td>VK_PRIOR</td></tr>
<tr><td>E0 4B</td><td>VK_LEFT</td></tr>
<tr><td>E0 4D</td><td>VK_RIGHT</td></tr>
<tr><td>E0 4F</td><td>VK_END</td></tr>
<tr><td>E0 50</td><td>VK_DOWN</td></tr>
<tr><td>E0 51</td><td>VK_NEXT</td></tr>
<tr><td>E0 52</td><td>VK_INSERT</td></tr>
<tr><td>E0 53</td><td>VK_DELETE</td></tr>
<tr><td>E0 5B</td><td>VK_LWIN</td></tr>
<tr><td>E0 5C</td><td>VK_RWIN</td></tr>
<tr><td>E0 5D</td><td>VK_APPS</td></tr>
<tr><td>E0 5F</td><td>VK_SLEEP</td></tr>
<tr><td>E0 65</td><td>VK_BROWSER_SEARCH</td></tr>
<tr><td>E0 66</td><td>VK_BROWSER_FAVORITES</td></tr>
<tr><td>E0 67</td><td>VK_BROWSER_REFRESH</td></tr>
<tr><td>E0 68</td><td>VK_BROWSER_STOP</td></tr>
<tr><td>E0 69</td><td>VK_BROWSER_FORWARD</td></tr>
<tr><td>E0 6A</td><td>VK_BROWSER_BACK</td></tr>
<tr><td>E0 6B</td><td>VK_LAUNCH_APP1</td></tr>
<tr><td>E0 6C</td><td>VK_LAUNCH_MAIL</td></tr>
<tr><td>E0 6D</td><td>VK_LAUNCH_MEDIA_SELECT</td></tr>
<tr><td>E1 1D</td><td>VK_PAUSE</td></tr>
</table></pre>
