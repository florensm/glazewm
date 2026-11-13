
### float: vs `morepeeks`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | morepeeks | 88.00 <sub>83.57-88.69</sub> |  | 98.66 <sub>97.87-102.40</sub> |  |
| input->first frame ms | morepeeks+keeppeeks | 87.69 <sub>85.92-91.63</sub> | -0.31 -0% ~ | 95.30 <sub>92.38-101.72</sub> | -3.37 -3% ~ |
| input->first frame ms | morepeeks+earlylanding+asynchandback+keeppeeks | 88.88 <sub>84.40-95.00</sub> | +0.88 +1% ~ | 98.05 <sub>96.49-100.32</sub> | -0.61 -1% ~ |
| command p50 ms | morepeeks | 63.85 <sub>62.58-64.83</sub> |  | 71.19 <sub>70.39-71.87</sub> |  |
| command p50 ms | morepeeks+keeppeeks | 60.52 <sub>60.14-65.67</sub> | -3.33 -5% ~ | 68.26 <sub>65.40-72.57</sub> | -2.94 -4% ~ |
| command p50 ms | morepeeks+earlylanding+asynchandback+keeppeeks | 64.56 <sub>60.50-68.25</sub> | +0.71 +1% ~ | 69.56 <sub>67.75-72.46</sub> | -1.63 -2% ~ |
| command ms (WM side) | morepeeks | 63.50 <sub>62.30-64.50</sub> |  | 70.85 <sub>70.00-71.50</sub> |  |
| command ms (WM side) | morepeeks+keeppeeks | 60.20 <sub>59.90-65.40</sub> | -3.30 -5% ~ | 67.80 <sub>65.10-72.20</sub> | -3.05 -4% ~ |
| command ms (WM side) | morepeeks+earlylanding+asynchandback+keeppeeks | 64.20 <sub>60.10-67.90</sub> | +0.70 +1% ~ | 69.25 <sub>67.40-72.20</sub> | -1.60 -2% ~ |
| frame p90 ms | morepeeks | 3.14 <sub>3.05-3.72</sub> |  | 4.38 <sub>3.84-4.57</sub> |  |
| frame p90 ms | morepeeks+keeppeeks | 3.12 <sub>2.98-3.53</sub> | -0.02 -1% ~ | 3.89 <sub>3.76-4.39</sub> | -0.50 -11% ~ |
| frame p90 ms | morepeeks+earlylanding+asynchandback+keeppeeks | 7.19 <sub>4.69-9.54</sub> | +4.05 +129% **sep** | 10.18 <sub>8.79-10.90</sub> | +5.79 +132% **sep** |
| interval p90 ms | morepeeks | 5.83 <sub>5.80-6.26</sub> |  | 6.13 <sub>5.80-6.56</sub> |  |
| interval p90 ms | morepeeks+keeppeeks | 5.80 <sub>5.79-5.81</sub> | -0.03 -1% ~ | 5.79 <sub>5.78-5.82</sub> | -0.34 -5% ~ |
| interval p90 ms | morepeeks+earlylanding+asynchandback+keeppeeks | 10.43 <sub>9.39-11.33</sub> | +4.59 +79% **sep** | 10.88 <sub>10.16-11.45</sub> | +4.74 +77% **sep** |
| interval p99 ms | morepeeks | 34.56 <sub>32.78-34.73</sub> |  | 38.57 <sub>36.90-39.60</sub> |  |
| interval p99 ms | morepeeks+keeppeeks | 33.66 <sub>33.26-37.05</sub> | -0.90 -3% ~ | 37.59 <sub>36.59-38.38</sub> | -0.98 -3% ~ |
| interval p99 ms | morepeeks+earlylanding+asynchandback+keeppeeks | 25.50 <sub>24.33-27.64</sub> | -9.06 -26% **sep** | 29.17 <sub>27.23-29.44</sub> | -9.40 -24% **sep** |
| interval max ms (median burst) | morepeeks | 34.56 <sub>32.78-34.73</sub> |  | 38.57 <sub>36.90-39.60</sub> |  |
| interval max ms (median burst) | morepeeks+keeppeeks | 33.66 <sub>33.26-37.05</sub> | -0.90 -3% ~ | 37.59 <sub>36.59-38.38</sub> | -0.98 -3% ~ |
| interval max ms (median burst) | morepeeks+earlylanding+asynchandback+keeppeeks | 25.50 <sub>24.33-27.64</sub> | -9.06 -26% **sep** | 29.17 <sub>27.23-29.44</sub> | -9.40 -24% **sep** |
| frames | morepeeks | 60.00 <sub>59.00-60.00</sub> |  | 59.00 <sub>59.00-59.00</sub> |  |
| frames | morepeeks+keeppeeks | 60.00 <sub>60.00-60.00</sub> | +0.00 +0% ~ | 59.00 <sub>59.00-59.00</sub> | +0.00 +0% ~ |
| frames | morepeeks+earlylanding+asynchandback+keeppeeks | 59.00 <sub>59.00-59.00</sub> | -1.00 -2% ~ | 58.00 <sub>58.00-59.00</sub> | -1.00 -2% ~ |
| peek_new ms (outside frames) | morepeeks | 16.85 <sub>16.40-17.20</sub> |  | 19.30 <sub>18.80-19.80</sub> |  |
| peek_new ms (outside frames) | morepeeks+keeppeeks | 14.90 <sub>14.70-15.20</sub> | -1.95 -12% **sep** | 17.00 <sub>16.10-17.50</sub> | -2.30 -12% **sep** |
| peek_new ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 14.60 <sub>14.20-15.60</sub> | -2.25 -13% **sep** | 16.80 <sub>16.10-17.10</sub> | -2.50 -13% **sep** |
| peek_cover ms (outside frames) | morepeeks | 11.70 <sub>11.50-11.80</sub> |  | 12.55 <sub>12.30-13.60</sub> |  |
| peek_cover ms (outside frames) | morepeeks+keeppeeks | 11.60 <sub>11.40-11.70</sub> | -0.10 -1% ~ | 12.45 <sub>11.80-13.20</sub> | -0.10 -1% ~ |
| peek_cover ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 12.00 <sub>11.60-12.10</sub> | +0.30 +3% ~ | 12.30 <sub>11.50-13.20</sub> | -0.25 -2% ~ |
| peek_handover ms (outside frames) | morepeeks | 10.20 <sub>9.50-14.10</sub> |  | 13.40 <sub>9.50-16.10</sub> |  |
| peek_handover ms (outside frames) | morepeeks+keeppeeks | 11.30 <sub>9.60-13.10</sub> | +1.10 +11% ~ | 12.70 <sub>11.10-13.40</sub> | -0.70 -5% ~ |
| peek_handover ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 11.70 <sub>10.20-13.10</sub> | +1.50 +15% ~ | 13.65 <sub>11.20-16.60</sub> | +0.25 +2% ~ |
| peek_raise ms (outside frames) | morepeeks | 1.75 <sub>1.40-2.20</sub> |  | 1.90 <sub>1.70-2.40</sub> |  |
| peek_raise ms (outside frames) | morepeeks+keeppeeks | 1.65 <sub>1.60-2.20</sub> | -0.10 -6% ~ | 2.00 <sub>1.70-2.40</sub> | +0.10 +5% ~ |
| peek_raise ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 1.90 <sub>1.50-2.50</sub> | +0.15 +9% ~ | 1.95 <sub>1.80-2.80</sub> | +0.05 +3% ~ |
| cloak ms (outside frames) | morepeeks | 18.20 <sub>16.80-19.80</sub> |  | 17.75 <sub>16.80-18.60</sub> |  |
| cloak ms (outside frames) | morepeeks+keeppeeks | 18.85 <sub>17.50-22.70</sub> | +0.65 +4% ~ | 18.55 <sub>18.10-20.60</sub> | +0.80 +5% ~ |
| cloak ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 18.75 <sub>18.50-24.20</sub> | +0.55 +3% ~ | 18.00 <sub>16.90-19.20</sub> | +0.25 +1% ~ |
| session_overlays ms (outside frames) | morepeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| session_overlays ms (outside frames) | morepeeks+keeppeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |

### resize: vs `morepeeks`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | morepeeks | 87.48 <sub>85.76-88.14</sub> |  | 99.69 <sub>97.33-101.38</sub> |  |
| input->first frame ms | morepeeks+keeppeeks | 87.16 <sub>84.64-88.41</sub> | -0.32 -0% ~ | 97.43 <sub>92.30-101.12</sub> | -2.26 -2% ~ |
| input->first frame ms | morepeeks+earlylanding+asynchandback+keeppeeks | 86.67 <sub>83.01-89.49</sub> | -0.81 -1% ~ | 97.96 <sub>96.14-100.47</sub> | -1.73 -2% ~ |
| command p50 ms | morepeeks | 63.79 <sub>61.52-64.49</sub> |  | 68.42 <sub>64.28-69.95</sub> |  |
| command p50 ms | morepeeks+keeppeeks | 60.30 <sub>58.72-62.60</sub> | -3.49 -5% ~ | 65.37 <sub>64.53-70.57</sub> | -3.05 -4% ~ |
| command p50 ms | morepeeks+earlylanding+asynchandback+keeppeeks | 59.74 <sub>58.42-63.61</sub> | -4.05 -6% ~ | 68.37 <sub>61.41-69.54</sub> | -0.05 -0% ~ |
| command ms (WM side) | morepeeks | 63.40 <sub>61.20-64.10</sub> |  | 68.05 <sub>64.00-69.60</sub> |  |
| command ms (WM side) | morepeeks+keeppeeks | 60.00 <sub>58.40-62.20</sub> | -3.40 -5% ~ | 65.05 <sub>64.20-70.20</sub> | -3.00 -4% ~ |
| command ms (WM side) | morepeeks+earlylanding+asynchandback+keeppeeks | 59.40 <sub>58.10-63.30</sub> | -4.00 -6% ~ | 67.90 <sub>61.10-69.10</sub> | -0.15 -0% ~ |
| frame p90 ms | morepeeks | 3.35 <sub>3.05-3.83</sub> |  | 4.22 <sub>3.83-4.39</sub> |  |
| frame p90 ms | morepeeks+keeppeeks | 3.34 <sub>3.18-3.73</sub> | -0.01 -0% ~ | 4.00 <sub>3.88-4.17</sub> | -0.22 -5% ~ |
| frame p90 ms | morepeeks+earlylanding+asynchandback+keeppeeks | 5.57 <sub>4.55-7.17</sub> | +2.22 +66% **sep** | 8.42 <sub>5.22-10.74</sub> | +4.19 +99% **sep** |
| interval p90 ms | morepeeks | 5.79 <sub>5.78-6.47</sub> |  | 6.27 <sub>5.84-6.55</sub> |  |
| interval p90 ms | morepeeks+keeppeeks | 5.80 <sub>5.79-5.81</sub> | +0.01 +0% ~ | 5.81 <sub>5.78-5.85</sub> | -0.46 -7% ~ |
| interval p90 ms | morepeeks+earlylanding+asynchandback+keeppeeks | 10.03 <sub>5.79-11.38</sub> | +4.24 +73% ~ | 9.75 <sub>8.16-13.43</sub> | +3.48 +56% **sep** |
| interval p99 ms | morepeeks | 47.91 <sub>41.78-56.79</sub> |  | 51.16 <sub>50.18-54.85</sub> |  |
| interval p99 ms | morepeeks+keeppeeks | 48.48 <sub>40.11-53.06</sub> | +0.58 +1% ~ | 52.09 <sub>45.76-58.64</sub> | +0.94 +2% ~ |
| interval p99 ms | morepeeks+earlylanding+asynchandback+keeppeeks | 27.99 <sub>26.64-28.94</sub> | -19.91 -42% **sep** | 33.28 <sub>28.97-34.02</sub> | -17.87 -35% **sep** |
| interval max ms (median burst) | morepeeks | 47.91 <sub>41.78-56.79</sub> |  | 51.16 <sub>50.18-54.85</sub> |  |
| interval max ms (median burst) | morepeeks+keeppeeks | 48.48 <sub>40.11-53.06</sub> | +0.58 +1% ~ | 52.09 <sub>45.76-58.64</sub> | +0.94 +2% ~ |
| interval max ms (median burst) | morepeeks+earlylanding+asynchandback+keeppeeks | 27.99 <sub>26.64-28.94</sub> | -19.91 -42% **sep** | 33.28 <sub>28.97-34.02</sub> | -17.87 -35% **sep** |
| frames | morepeeks | 61.00 <sub>60.00-61.00</sub> |  | 59.00 <sub>58.00-59.00</sub> |  |
| frames | morepeeks+keeppeeks | 60.00 <sub>60.00-60.00</sub> | -1.00 -2% ~ | 59.00 <sub>59.00-60.00</sub> | +0.00 +0% ~ |
| frames | morepeeks+earlylanding+asynchandback+keeppeeks | 59.50 <sub>59.00-61.00</sub> | -1.50 -2% ~ | 58.00 <sub>58.00-58.00</sub> | -1.00 -2% ~ |
| peek_new ms (outside frames) | morepeeks | 17.40 <sub>16.90-17.90</sub> |  | 20.25 <sub>20.00-20.80</sub> |  |
| peek_new ms (outside frames) | morepeeks+keeppeeks | 15.30 <sub>14.90-15.60</sub> | -2.10 -12% **sep** | 17.95 <sub>17.70-18.50</sub> | -2.30 -11% **sep** |
| peek_new ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 14.75 <sub>14.30-15.30</sub> | -2.65 -15% **sep** | 17.50 <sub>16.40-17.90</sub> | -2.75 -14% **sep** |
| peek_cover ms (outside frames) | morepeeks | 10.30 <sub>10.10-10.70</sub> |  | 11.15 <sub>10.60-11.80</sub> |  |
| peek_cover ms (outside frames) | morepeeks+keeppeeks | 10.40 <sub>10.30-10.70</sub> | +0.10 +1% ~ | 10.95 <sub>10.70-11.40</sub> | -0.20 -2% ~ |
| peek_cover ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 10.15 <sub>9.80-10.60</sub> | -0.15 -1% ~ | 10.70 <sub>10.60-10.90</sub> | -0.45 -4% ~ |
| peek_handover ms (outside frames) | morepeeks | 12.65 <sub>11.20-13.10</sub> |  | 14.25 <sub>10.90-17.10</sub> |  |
| peek_handover ms (outside frames) | morepeeks+keeppeeks | 11.65 <sub>9.80-12.80</sub> | -1.00 -8% ~ | 12.50 <sub>12.30-14.80</sub> | -1.75 -12% ~ |
| peek_handover ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 12.40 <sub>10.40-12.90</sub> | -0.25 -2% ~ | 12.50 <sub>12.40-16.90</sub> | -1.75 -12% ~ |
| peek_raise ms (outside frames) | morepeeks | 1.80 <sub>1.50-1.90</sub> |  | 1.75 <sub>1.70-2.60</sub> |  |
| peek_raise ms (outside frames) | morepeeks+keeppeeks | 1.75 <sub>1.20-2.30</sub> | -0.05 -3% ~ | 2.00 <sub>1.40-2.20</sub> | +0.25 +14% ~ |
| peek_raise ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 1.85 <sub>1.30-2.50</sub> | +0.05 +3% ~ | 2.05 <sub>1.70-2.40</sub> | +0.30 +17% ~ |
| cloak ms (outside frames) | morepeeks | 18.60 <sub>17.40-20.10</sub> |  | 16.85 <sub>13.60-20.80</sub> |  |
| cloak ms (outside frames) | morepeeks+keeppeeks | 18.20 <sub>15.40-23.90</sub> | -0.40 -2% ~ | 18.60 <sub>13.40-20.40</sub> | +1.75 +10% ~ |
| cloak ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 16.25 <sub>13.70-18.30</sub> | -2.35 -13% ~ | 18.65 <sub>17.10-20.00</sub> | +1.80 +11% ~ |
| session_overlays ms (outside frames) | morepeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| session_overlays ms (outside frames) | morepeeks+keeppeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | morepeeks+earlylanding+asynchandback+keeppeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
