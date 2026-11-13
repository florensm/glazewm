
### resize: vs `nopeeks`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | nopeeks | 34.98 <sub>32.66-37.82</sub> |  | 51.93 <sub>49.78-53.70</sub> |  |
| input->first frame ms | nopeeks+nokeepoverlays | 39.69 <sub>36.55-42.58</sub> | +4.70 +13% ~ | 54.75 <sub>52.74-57.10</sub> | +2.82 +5% ~ |
| command p50 ms | nopeeks | 30.52 <sub>30.07-31.07</sub> |  | 42.85 <sub>42.24-44.98</sub> |  |
| command p50 ms | nopeeks+nokeepoverlays | 35.88 <sub>30.54-37.70</sub> | +5.36 +18% ~ | 49.10 <sub>45.17-52.66</sub> | +6.25 +15% **sep** |
| command ms (WM side) | nopeeks | 30.15 <sub>29.70-30.60</sub> |  | 42.45 <sub>41.50-44.50</sub> |  |
| command ms (WM side) | nopeeks+nokeepoverlays | 35.55 <sub>30.20-37.40</sub> | +5.40 +18% ~ | 48.70 <sub>44.80-52.20</sub> | +6.25 +15% **sep** |
| frame p90 ms | nopeeks | 3.79 <sub>3.58-4.51</sub> |  | 5.82 <sub>4.68-6.27</sub> |  |
| frame p90 ms | nopeeks+nokeepoverlays | 3.78 <sub>3.52-4.05</sub> | -0.01 -0% ~ | 4.92 <sub>4.81-5.65</sub> | -0.91 -16% ~ |
| interval p90 ms | nopeeks | 5.78 <sub>5.77-5.79</sub> |  | 6.42 <sub>6.17-6.85</sub> |  |
| interval p90 ms | nopeeks+nokeepoverlays | 5.78 <sub>5.77-5.78</sub> | +0.00 +0% ~ | 5.97 <sub>5.84-6.65</sub> | -0.45 -7% ~ |
| interval p99 ms | nopeeks | 14.51 <sub>14.27-15.91</sub> |  | 27.35 <sub>24.95-30.38</sub> |  |
| interval p99 ms | nopeeks+nokeepoverlays | 14.09 <sub>13.57-14.62</sub> | -0.42 -3% ~ | 27.35 <sub>26.94-27.86</sub> | +0.00 +0% ~ |
| interval max ms (median burst) | nopeeks | 14.51 <sub>14.27-15.91</sub> |  | 27.35 <sub>24.95-30.38</sub> |  |
| interval max ms (median burst) | nopeeks+nokeepoverlays | 14.09 <sub>13.57-14.62</sub> | -0.42 -3% ~ | 27.35 <sub>26.94-27.86</sub> | +0.00 +0% ~ |
| frames | nopeeks | 62.00 <sub>62.00-62.00</sub> |  | 61.00 <sub>60.00-62.00</sub> |  |
| frames | nopeeks+nokeepoverlays | 62.00 <sub>62.00-63.00</sub> | +0.00 +0% ~ | 61.50 <sub>61.00-62.00</sub> | +0.50 +1% ~ |
| peek_new ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_new ms (outside frames) | nopeeks+nokeepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_cover ms (outside frames) | nopeeks+nokeepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_handover ms (outside frames) | nopeeks+nokeepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_raise ms (outside frames) | nopeeks+nokeepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| cloak ms (outside frames) | nopeeks | 18.25 <sub>17.80-18.70</sub> |  | 25.75 <sub>24.30-26.40</sub> |  |
| cloak ms (outside frames) | nopeeks+nokeepoverlays | 21.15 <sub>19.30-21.20</sub> | +2.90 +16% **sep** | 26.90 <sub>25.20-30.10</sub> | +1.15 +4% ~ |
| session_overlays ms (outside frames) | nopeeks | 7.10 <sub>7.10-7.50</sub> |  | 9.85 <sub>8.10-11.00</sub> |  |
| session_overlays ms (outside frames) | nopeeks+nokeepoverlays | 10.80 <sub>10.30-11.00</sub> | +3.70 +52% **sep** | 15.15 <sub>14.20-16.30</sub> | +5.30 +54% **sep** |

### resize: vs `default`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | default | 82.22 <sub>78.22-82.53</sub> |  | 103.50 <sub>98.89-105.44</sub> |  |
| input->first frame ms | noasynchandback | 77.58 <sub>72.84-81.89</sub> | -4.64 -6% ~ | 96.44 <sub>95.10-104.78</sub> | -7.06 -7% ~ |
| command p50 ms | default | 47.73 <sub>46.42-49.30</sub> |  | 67.63 <sub>67.34-72.03</sub> |  |
| command p50 ms | noasynchandback | 45.49 <sub>41.70-48.90</sub> | -2.23 -5% ~ | 66.91 <sub>63.22-68.61</sub> | -0.72 -1% ~ |
| command ms (WM side) | default | 47.35 <sub>46.10-49.00</sub> |  | 67.25 <sub>66.50-71.60</sub> |  |
| command ms (WM side) | noasynchandback | 45.20 <sub>41.40-48.70</sub> | -2.15 -5% ~ | 66.55 <sub>62.70-68.20</sub> | -0.70 -1% ~ |
| frame p90 ms | default | 2.96 <sub>2.72-3.11</sub> |  | 3.35 <sub>3.28-3.46</sub> |  |
| frame p90 ms | noasynchandback | 2.79 <sub>2.73-2.86</sub> | -0.16 -5% ~ | 3.39 <sub>3.29-3.49</sub> | +0.03 +1% ~ |
| interval p90 ms | default | 5.80 <sub>5.79-5.83</sub> |  | 5.88 <sub>5.83-6.04</sub> |  |
| interval p90 ms | noasynchandback | 5.80 <sub>5.79-5.81</sub> | -0.00 -0% ~ | 5.90 <sub>5.87-6.17</sub> | +0.03 +1% ~ |
| interval p99 ms | default | 35.62 <sub>33.93-36.06</sub> |  | 53.16 <sub>51.16-54.25</sub> |  |
| interval p99 ms | noasynchandback | 47.69 <sub>45.44-53.33</sub> | +12.06 +34% **sep** | 59.59 <sub>56.94-62.57</sub> | +6.43 +12% **sep** |
| interval max ms (median burst) | default | 35.62 <sub>33.93-36.06</sub> |  | 53.16 <sub>51.16-54.25</sub> |  |
| interval max ms (median burst) | noasynchandback | 47.69 <sub>45.44-53.33</sub> | +12.06 +34% **sep** | 59.59 <sub>56.94-62.57</sub> | +6.43 +12% **sep** |
| frames | default | 59.00 <sub>59.00-60.00</sub> |  | 59.00 <sub>58.00-59.00</sub> |  |
| frames | noasynchandback | 59.00 <sub>59.00-60.00</sub> | +0.00 +0% ~ | 59.00 <sub>58.00-59.00</sub> | +0.00 +0% ~ |
| peek_new ms (outside frames) | default | 5.65 <sub>5.60-5.90</sub> |  | 7.25 <sub>6.90-7.40</sub> |  |
| peek_new ms (outside frames) | noasynchandback | 5.70 <sub>5.60-5.90</sub> | +0.05 +1% ~ | 7.10 <sub>7.00-7.50</sub> | -0.15 -2% ~ |
| peek_cover ms (outside frames) | default | 8.05 <sub>7.70-8.20</sub> |  | 9.90 <sub>9.60-10.40</sub> |  |
| peek_cover ms (outside frames) | noasynchandback | 8.10 <sub>7.90-8.10</sub> | +0.05 +1% ~ | 10.00 <sub>9.60-10.30</sub> | +0.10 +1% ~ |
| peek_handover ms (outside frames) | default | 2.65 <sub>2.50-2.70</sub> |  | 3.45 <sub>3.30-3.80</sub> |  |
| peek_handover ms (outside frames) | noasynchandback | 2.75 <sub>2.70-3.10</sub> | +0.10 +4% ~ | 3.00 <sub>2.80-3.10</sub> | -0.45 -13% **sep** |
| peek_raise ms (outside frames) | default | 3.55 <sub>2.20-3.80</sub> |  | 4.50 <sub>3.90-4.60</sub> |  |
| peek_raise ms (outside frames) | noasynchandback | 3.50 <sub>3.10-5.70</sub> | -0.05 -1% ~ | 5.25 <sub>4.30-8.20</sub> | +0.75 +17% ~ |
| cloak ms (outside frames) | default | 21.00 <sub>20.70-22.60</sub> |  | 32.45 <sub>32.20-35.10</sub> |  |
| cloak ms (outside frames) | noasynchandback | 19.60 <sub>16.40-21.40</sub> | -1.40 -7% ~ | 31.25 <sub>29.90-35.90</sub> | -1.20 -4% ~ |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| session_overlays ms (outside frames) | noasynchandback | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
