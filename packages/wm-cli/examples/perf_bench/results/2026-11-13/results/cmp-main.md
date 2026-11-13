
### move: vs `main`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | main | 13.95 <sub>13.28-15.06</sub> |  | 14.16 <sub>13.27-15.17</sub> |  |
| input->first frame ms | nopeeks | 16.45 <sub>14.71-17.71</sub> | +2.50 +18% ~ | 16.34 <sub>15.60-16.75</sub> | +2.18 +15% **sep** |
| input->first frame ms | default | 19.97 <sub>19.89-20.08</sub> | +6.02 +43% **sep** | 21.66 <sub>19.47-22.07</sub> | +7.49 +53% **sep** |
| command p50 ms | main | 13.35 <sub>12.76-14.09</sub> |  | 13.21 <sub>12.53-14.13</sub> |  |
| command p50 ms | nopeeks | 15.89 <sub>14.22-17.20</sub> | +2.54 +19% **sep** | 15.52 <sub>14.91-15.92</sub> | +2.30 +17% **sep** |
| command p50 ms | default | 19.32 <sub>19.26-19.42</sub> | +5.98 +45% **sep** | 20.69 <sub>18.56-21.16</sub> | +7.47 +57% **sep** |
| command ms (WM side) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| command ms (WM side) | nopeeks | 15.55 <sub>13.90-16.90</sub> | +15.55 **sep** | 15.25 <sub>14.60-15.60</sub> | +15.25 **sep** |
| command ms (WM side) | default | 18.95 <sub>18.90-19.10</sub> | +18.95 **sep** | 20.40 <sub>18.30-20.90</sub> | +20.40 **sep** |
| frame p90 ms | main | 1.32 <sub>1.32-1.35</sub> |  | 1.42 <sub>1.41-1.47</sub> |  |
| frame p90 ms | nopeeks | 1.31 <sub>1.29-1.33</sub> | -0.01 -1% ~ | 1.37 <sub>1.35-1.37</sub> | -0.06 -4% **sep** |
| frame p90 ms | default | 0.89 <sub>0.82-0.90</sub> | -0.43 -33% **sep** | 0.96 <sub>0.95-0.99</sub> | -0.46 -33% **sep** |
| interval p90 ms | main | 5.73 <sub>5.73-5.74</sub> |  | 5.74 <sub>5.74-5.75</sub> |  |
| interval p90 ms | nopeeks | 5.74 <sub>5.74-5.74</sub> | +0.01 +0% ~ | 5.74 <sub>5.74-5.75</sub> | +0.00 +0% ~ |
| interval p90 ms | default | 5.75 <sub>5.74-5.76</sub> | +0.02 +0% ~ | 5.75 <sub>5.75-5.76</sub> | +0.01 +0% ~ |
| interval p99 ms | main | 7.37 <sub>7.29-7.97</sub> |  | 9.00 <sub>7.80-9.58</sub> |  |
| interval p99 ms | nopeeks | 7.92 <sub>7.81-8.18</sub> | +0.55 +7% ~ | 8.92 <sub>8.57-9.33</sub> | -0.08 -1% ~ |
| interval p99 ms | default | 10.79 <sub>10.70-10.91</sub> | +3.42 +46% **sep** | 12.23 <sub>11.72-12.57</sub> | +3.22 +36% **sep** |
| interval max ms (median burst) | main | 7.37 <sub>7.29-7.97</sub> |  | 9.00 <sub>7.80-9.58</sub> |  |
| interval max ms (median burst) | nopeeks | 7.92 <sub>7.81-8.18</sub> | +0.55 +7% ~ | 8.92 <sub>8.57-9.33</sub> | -0.08 -1% ~ |
| interval max ms (median burst) | default | 10.79 <sub>10.70-10.91</sub> | +3.42 +46% **sep** | 12.23 <sub>11.72-12.57</sub> | +3.22 +36% **sep** |
| frames | main | 63.00 <sub>63.00-63.00</sub> |  | 63.00 <sub>63.00-63.00</sub> |  |
| frames | nopeeks | 62.00 <sub>62.00-62.00</sub> | -1.00 -2% **sep** | 62.00 <sub>62.00-62.00</sub> | -1.00 -2% **sep** |
| frames | default | 63.00 <sub>63.00-63.00</sub> | +0.00 +0% ~ | 63.00 <sub>63.00-63.00</sub> | +0.00 +0% ~ |
| peek_new ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_new ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_new ms (outside frames) | default | 5.50 <sub>5.30-5.70</sub> | +5.50 **sep** | 6.30 <sub>5.80-6.50</sub> | +6.30 **sep** |
| peek_cover ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_cover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | default | 1.05 <sub>1.00-1.10</sub> | +1.05 **sep** | 1.10 <sub>1.10-1.20</sub> | +1.10 **sep** |
| peek_handover ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_handover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | default | 2.70 <sub>2.50-2.80</sub> | +2.70 **sep** | 3.05 <sub>3.00-3.30</sub> | +3.05 **sep** |
| peek_raise ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_raise ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | default | 0.40 <sub>0.40-0.40</sub> | +0.40 **sep** | 0.40 <sub>0.30-0.40</sub> | +0.40 **sep** |
| cloak ms (outside frames) | main | 8.65 <sub>7.80-9.50</sub> |  | 8.50 <sub>7.60-9.10</sub> |  |
| cloak ms (outside frames) | nopeeks | 9.65 <sub>8.40-10.70</sub> | +1.00 +12% ~ | 9.25 <sub>8.80-9.60</sub> | +0.75 +9% ~ |
| cloak ms (outside frames) | default | 7.55 <sub>7.20-7.90</sub> | -1.10 -13% ~ | 7.45 <sub>6.00-7.70</sub> | -1.05 -12% ~ |
| session_overlays ms (outside frames) | main | 1.95 <sub>1.80-2.40</sub> |  | 2.15 <sub>2.00-2.20</sub> |  |
| session_overlays ms (outside frames) | nopeeks | 3.55 <sub>3.40-3.70</sub> | +1.60 +82% **sep** | 3.30 <sub>3.20-4.00</sub> | +1.15 +53% **sep** |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | -1.95 -100% **sep** | 0.00 <sub>0.00-0.00</sub> | -2.15 -100% **sep** |

### resize: vs `main`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | main | 31.81 <sub>31.26-32.60</sub> |  | 36.55 <sub>35.30-39.66</sub> |  |
| input->first frame ms | nopeeks | 43.70 <sub>42.90-44.81</sub> | +11.89 +37% **sep** | 48.41 <sub>46.72-49.40</sub> | +11.85 +32% **sep** |
| input->first frame ms | default | 86.60 <sub>85.67-87.45</sub> | +54.78 +172% **sep** | 98.33 <sub>95.10-99.25</sub> | +61.78 +169% **sep** |
| command p50 ms | main | 27.55 <sub>26.97-27.88</sub> |  | 30.66 <sub>29.78-32.31</sub> |  |
| command p50 ms | nopeeks | 39.94 <sub>39.37-40.81</sub> | +12.39 +45% **sep** | 43.41 <sub>41.79-44.55</sub> | +12.76 +42% **sep** |
| command p50 ms | default | 60.71 <sub>59.94-61.22</sub> | +33.16 +120% **sep** | 68.36 <sub>67.95-69.68</sub> | +37.70 +123% **sep** |
| command ms (WM side) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| command ms (WM side) | nopeeks | 39.60 <sub>39.10-40.40</sub> | +39.60 **sep** | 43.10 <sub>41.50-44.20</sub> | +43.10 **sep** |
| command ms (WM side) | default | 60.30 <sub>59.60-60.90</sub> | +60.30 **sep** | 68.05 <sub>67.60-69.00</sub> | +68.05 **sep** |
| frame p90 ms | main | 4.24 <sub>3.76-4.41</sub> |  | 4.63 <sub>4.31-5.14</sub> |  |
| frame p90 ms | nopeeks | 4.54 <sub>4.21-4.74</sub> | +0.30 +7% ~ | 4.95 <sub>4.71-5.15</sub> | +0.32 +7% ~ |
| frame p90 ms | default | 3.26 <sub>3.10-3.45</sub> | -0.98 -23% **sep** | 4.04 <sub>3.97-4.48</sub> | -0.58 -13% ~ |
| interval p90 ms | main | 5.77 <sub>5.76-5.77</sub> |  | 5.76 <sub>5.76-5.79</sub> |  |
| interval p90 ms | nopeeks | 5.78 <sub>5.77-5.80</sub> | +0.01 +0% ~ | 5.78 <sub>5.77-5.80</sub> | +0.01 +0% ~ |
| interval p90 ms | default | 5.78 <sub>5.78-5.98</sub> | +0.01 +0% **sep** | 5.87 <sub>5.79-5.98</sub> | +0.10 +2% ~ |
| interval p99 ms | main | 25.53 <sub>24.07-25.89</sub> |  | 27.49 <sub>25.72-30.99</sub> |  |
| interval p99 ms | nopeeks | 17.58 <sub>16.76-18.44</sub> | -7.95 -31% **sep** | 19.71 <sub>19.23-20.82</sub> | -7.78 -28% **sep** |
| interval p99 ms | default | 49.26 <sub>47.69-52.49</sub> | +23.73 +93% **sep** | 57.59 <sub>54.81-66.10</sub> | +30.10 +110% **sep** |
| interval max ms (median burst) | main | 25.53 <sub>24.07-25.89</sub> |  | 27.49 <sub>25.72-30.99</sub> |  |
| interval max ms (median burst) | nopeeks | 17.58 <sub>16.76-18.44</sub> | -7.95 -31% **sep** | 19.71 <sub>19.23-20.82</sub> | -7.78 -28% **sep** |
| interval max ms (median burst) | default | 49.26 <sub>47.69-52.49</sub> | +23.73 +93% **sep** | 57.59 <sub>54.81-66.10</sub> | +30.10 +110% **sep** |
| frames | main | 63.00 <sub>63.00-63.00</sub> |  | 63.00 <sub>63.00-63.00</sub> |  |
| frames | nopeeks | 62.00 <sub>62.00-63.00</sub> | -1.00 -2% ~ | 62.00 <sub>62.00-62.00</sub> | -1.00 -2% **sep** |
| frames | default | 60.00 <sub>60.00-60.00</sub> | -3.00 -5% **sep** | 59.50 <sub>59.00-60.00</sub> | -3.50 -6% **sep** |
| peek_new ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_new ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_new ms (outside frames) | default | 16.80 <sub>16.70-17.40</sub> | +16.80 **sep** | 19.75 <sub>19.20-20.10</sub> | +19.75 **sep** |
| peek_cover ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_cover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | default | 9.95 <sub>9.80-10.20</sub> | +9.95 **sep** | 10.90 <sub>10.60-11.20</sub> | +10.90 **sep** |
| peek_handover ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_handover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | default | 11.75 <sub>11.30-13.00</sub> | +11.75 **sep** | 13.95 <sub>12.50-14.40</sub> | +13.95 **sep** |
| peek_raise ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_raise ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | default | 1.75 <sub>1.60-1.80</sub> | +1.75 **sep** | 2.15 <sub>2.00-2.40</sub> | +2.15 **sep** |
| cloak ms (outside frames) | main | 14.80 <sub>14.30-15.60</sub> |  | 18.60 <sub>18.00-19.00</sub> |  |
| cloak ms (outside frames) | nopeeks | 20.95 <sub>20.30-21.50</sub> | +6.15 +42% **sep** | 21.25 <sub>20.30-22.10</sub> | +2.65 +14% **sep** |
| cloak ms (outside frames) | default | 15.50 <sub>15.10-18.50</sub> | +0.70 +5% ~ | 16.90 <sub>16.20-17.90</sub> | -1.70 -9% **sep** |
| session_overlays ms (outside frames) | main | 8.15 <sub>7.20-8.60</sub> |  | 7.10 <sub>6.70-8.60</sub> |  |
| session_overlays ms (outside frames) | nopeeks | 14.75 <sub>13.80-15.60</sub> | +6.60 +81% **sep** | 17.75 <sub>16.30-18.10</sub> | +10.65 +150% **sep** |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | -8.15 -100% **sep** | 0.00 <sub>0.00-0.00</sub> | -7.10 -100% **sep** |
