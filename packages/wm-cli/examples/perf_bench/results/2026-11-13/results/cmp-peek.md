
### move: vs `default`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | default | 19.89 <sub>19.70-20.14</sub> |  | 21.48 <sub>21.27-21.86</sub> |  |
| input->first frame ms | keeppeeks | 19.56 <sub>19.35-21.11</sub> | -0.33 -2% ~ | 20.73 <sub>20.25-20.84</sub> | -0.75 -3% **sep** |
| input->first frame ms | asynchandback | 20.52 <sub>19.78-20.90</sub> | +0.62 +3% ~ | 21.50 <sub>20.10-21.92</sub> | +0.02 +0% ~ |
| input->first frame ms | asynclanding | 19.55 <sub>19.44-19.82</sub> | -0.34 -2% ~ | 21.43 <sub>19.76-22.24</sub> | -0.05 -0% ~ |
| input->first frame ms | earlylanding | 20.69 <sub>20.26-20.92</sub> | +0.80 +4% **sep** | 21.47 <sub>20.47-21.96</sub> | -0.01 -0% ~ |
| input->first frame ms | earlylanding+asynchandback+keeppeeks | 19.25 <sub>18.81-19.60</sub> | -0.64 -3% **sep** | 20.52 <sub>20.16-21.01</sub> | -0.96 -4% **sep** |
| command p50 ms | default | 19.24 <sub>19.01-19.65</sub> |  | 20.46 <sub>20.28-20.98</sub> |  |
| command p50 ms | keeppeeks | 18.98 <sub>18.75-21.96</sub> | -0.26 -1% ~ | 19.95 <sub>19.69-20.18</sub> | -0.51 -2% **sep** |
| command p50 ms | asynchandback | 19.75 <sub>18.99-19.93</sub> | +0.51 +3% ~ | 20.34 <sub>19.24-20.89</sub> | -0.12 -1% ~ |
| command p50 ms | asynclanding | 18.69 <sub>17.96-19.40</sub> | -0.55 -3% ~ | 20.65 <sub>18.89-21.03</sub> | +0.19 +1% ~ |
| command p50 ms | earlylanding | 19.88 <sub>19.29-20.21</sub> | +0.64 +3% ~ | 20.44 <sub>19.23-20.93</sub> | -0.02 -0% ~ |
| command p50 ms | earlylanding+asynchandback+keeppeeks | 18.64 <sub>18.24-18.97</sub> | -0.60 -3% **sep** | 19.93 <sub>19.62-20.41</sub> | -0.53 -3% ~ |
| command ms (WM side) | default | 18.95 <sub>18.70-19.30</sub> |  | 20.15 <sub>20.00-20.70</sub> |  |
| command ms (WM side) | keeppeeks | 18.75 <sub>18.40-20.40</sub> | -0.20 -1% ~ | 19.70 <sub>19.40-19.90</sub> | -0.45 -2% **sep** |
| command ms (WM side) | asynchandback | 19.45 <sub>18.70-19.60</sub> | +0.50 +3% ~ | 20.05 <sub>19.00-20.60</sub> | -0.10 -0% ~ |
| command ms (WM side) | asynclanding | 18.40 <sub>17.60-19.10</sub> | -0.55 -3% ~ | 20.40 <sub>18.60-20.80</sub> | +0.25 +1% ~ |
| command ms (WM side) | earlylanding | 19.60 <sub>19.00-19.90</sub> | +0.65 +3% ~ | 20.15 <sub>19.00-20.60</sub> | +0.00 +0% ~ |
| command ms (WM side) | earlylanding+asynchandback+keeppeeks | 18.35 <sub>17.90-18.70</sub> | -0.60 -3% ~ | 19.65 <sub>19.40-20.20</sub> | -0.50 -2% ~ |
| frame p90 ms | default | 0.85 <sub>0.60-0.89</sub> |  | 0.96 <sub>0.93-0.99</sub> |  |
| frame p90 ms | keeppeeks | 0.51 <sub>0.44-0.53</sub> | -0.34 -40% **sep** | 0.48 <sub>0.48-0.49</sub> | -0.47 -49% **sep** |
| frame p90 ms | asynchandback | 0.86 <sub>0.84-0.88</sub> | +0.01 +1% ~ | 0.92 <sub>0.90-0.95</sub> | -0.04 -4% ~ |
| frame p90 ms | asynclanding | 0.85 <sub>0.59-0.91</sub> | -0.01 -1% ~ | 0.87 <sub>0.70-0.97</sub> | -0.09 -9% ~ |
| frame p90 ms | earlylanding | 0.89 <sub>0.83-0.89</sub> | +0.03 +4% ~ | 0.91 <sub>0.89-0.95</sub> | -0.05 -5% ~ |
| frame p90 ms | earlylanding+asynchandback+keeppeeks | 0.49 <sub>0.46-0.51</sub> | -0.36 -43% **sep** | 0.48 <sub>0.48-0.49</sub> | -0.47 -49% **sep** |
| interval p90 ms | default | 5.75 <sub>5.74-5.75</sub> |  | 5.75 <sub>5.74-5.76</sub> |  |
| interval p90 ms | keeppeeks | 5.74 <sub>5.74-5.75</sub> | -0.01 -0% ~ | 5.75 <sub>5.75-5.76</sub> | -0.00 -0% ~ |
| interval p90 ms | asynchandback | 5.75 <sub>5.74-5.75</sub> | +0.00 +0% ~ | 5.74 <sub>5.74-5.76</sub> | -0.01 -0% ~ |
| interval p90 ms | asynclanding | 5.75 <sub>5.74-5.76</sub> | +0.00 +0% ~ | 5.76 <sub>5.76-5.76</sub> | +0.00 +0% ~ |
| interval p90 ms | earlylanding | 5.75 <sub>5.74-5.76</sub> | +0.00 +0% ~ | 5.75 <sub>5.74-5.75</sub> | -0.01 -0% ~ |
| interval p90 ms | earlylanding+asynchandback+keeppeeks | 5.74 <sub>5.74-5.74</sub> | -0.01 -0% ~ | 5.75 <sub>5.74-5.76</sub> | +0.00 +0% ~ |
| interval p99 ms | default | 11.20 <sub>10.76-11.53</sub> |  | 12.31 <sub>12.07-12.57</sub> |  |
| interval p99 ms | keeppeeks | 10.00 <sub>9.65-12.35</sub> | -1.20 -11% ~ | 11.35 <sub>11.10-12.48</sub> | -0.96 -8% ~ |
| interval p99 ms | asynchandback | 10.85 <sub>10.56-11.31</sub> | -0.35 -3% ~ | 12.32 <sub>11.69-13.60</sub> | +0.01 +0% ~ |
| interval p99 ms | asynclanding | 11.20 <sub>10.46-11.75</sub> | -0.01 -0% ~ | 12.55 <sub>11.57-13.75</sub> | +0.24 +2% ~ |
| interval p99 ms | earlylanding | 10.51 <sub>10.05-10.89</sub> | -0.69 -6% ~ | 12.51 <sub>11.53-13.37</sub> | +0.20 +2% ~ |
| interval p99 ms | earlylanding+asynchandback+keeppeeks | 10.32 <sub>9.84-10.77</sub> | -0.88 -8% ~ | 12.66 <sub>11.61-13.03</sub> | +0.35 +3% ~ |
| interval max ms (median burst) | default | 11.20 <sub>10.76-11.53</sub> |  | 12.31 <sub>12.07-12.57</sub> |  |
| interval max ms (median burst) | keeppeeks | 10.00 <sub>9.65-12.35</sub> | -1.20 -11% ~ | 11.35 <sub>11.10-12.48</sub> | -0.96 -8% ~ |
| interval max ms (median burst) | asynchandback | 10.85 <sub>10.56-11.31</sub> | -0.35 -3% ~ | 12.32 <sub>11.69-13.60</sub> | +0.01 +0% ~ |
| interval max ms (median burst) | asynclanding | 11.20 <sub>10.46-11.75</sub> | -0.01 -0% ~ | 12.55 <sub>11.57-13.75</sub> | +0.24 +2% ~ |
| interval max ms (median burst) | earlylanding | 10.51 <sub>10.05-10.89</sub> | -0.69 -6% ~ | 12.51 <sub>11.53-13.37</sub> | +0.20 +2% ~ |
| interval max ms (median burst) | earlylanding+asynchandback+keeppeeks | 10.32 <sub>9.84-10.77</sub> | -0.88 -8% ~ | 12.66 <sub>11.61-13.03</sub> | +0.35 +3% ~ |
| frames | default | 63.00 <sub>63.00-63.00</sub> |  | 63.00 <sub>63.00-63.00</sub> |  |
| frames | keeppeeks | 63.00 <sub>63.00-64.00</sub> | +0.00 +0% ~ | 63.00 <sub>63.00-63.00</sub> | +0.00 +0% ~ |
| frames | asynchandback | 63.00 <sub>63.00-63.00</sub> | +0.00 +0% ~ | 63.00 <sub>63.00-64.00</sub> | +0.00 +0% ~ |
| frames | asynclanding | 63.00 <sub>63.00-64.00</sub> | +0.00 +0% ~ | 63.00 <sub>63.00-63.00</sub> | +0.00 +0% ~ |
| frames | earlylanding | 63.00 <sub>63.00-63.00</sub> | +0.00 +0% ~ | 63.00 <sub>63.00-63.00</sub> | +0.00 +0% ~ |
| frames | earlylanding+asynchandback+keeppeeks | 63.00 <sub>63.00-63.00</sub> | +0.00 +0% ~ | 63.00 <sub>63.00-63.00</sub> | +0.00 +0% ~ |
| peek_new ms (outside frames) | default | 5.30 <sub>5.20-5.40</sub> |  | 6.05 <sub>5.80-6.70</sub> |  |
| peek_new ms (outside frames) | keeppeeks | 4.95 <sub>4.90-5.30</sub> | -0.35 -7% ~ | 5.00 <sub>4.80-5.70</sub> | -1.05 -17% **sep** |
| peek_new ms (outside frames) | asynchandback | 5.50 <sub>5.40-5.60</sub> | +0.20 +4% ~ | 5.80 <sub>5.40-6.20</sub> | -0.25 -4% ~ |
| peek_new ms (outside frames) | asynclanding | 5.35 <sub>5.30-5.60</sub> | +0.05 +1% ~ | 5.80 <sub>5.40-6.20</sub> | -0.25 -4% ~ |
| peek_new ms (outside frames) | earlylanding | 5.50 <sub>5.30-5.70</sub> | +0.20 +4% ~ | 5.85 <sub>5.60-6.60</sub> | -0.20 -3% ~ |
| peek_new ms (outside frames) | earlylanding+asynchandback+keeppeeks | 5.10 <sub>4.90-5.20</sub> | -0.20 -4% ~ | 5.55 <sub>5.10-5.80</sub> | -0.50 -8% ~ |
| peek_cover ms (outside frames) | default | 1.05 <sub>1.00-1.10</sub> |  | 1.15 <sub>1.10-1.20</sub> |  |
| peek_cover ms (outside frames) | keeppeeks | 1.05 <sub>1.00-1.10</sub> | +0.00 +0% ~ | 1.10 <sub>1.00-1.20</sub> | -0.05 -4% ~ |
| peek_cover ms (outside frames) | asynchandback | 1.10 <sub>1.00-1.20</sub> | +0.05 +5% ~ | 1.10 <sub>1.10-1.20</sub> | -0.05 -4% ~ |
| peek_cover ms (outside frames) | asynclanding | 1.05 <sub>1.00-1.10</sub> | +0.00 +0% ~ | 1.10 <sub>1.00-1.20</sub> | -0.05 -4% ~ |
| peek_cover ms (outside frames) | earlylanding | 1.05 <sub>1.00-1.10</sub> | +0.00 +0% ~ | 1.10 <sub>1.10-1.20</sub> | -0.05 -4% ~ |
| peek_cover ms (outside frames) | earlylanding+asynchandback+keeppeeks | 1.00 <sub>1.00-1.10</sub> | -0.05 -5% ~ | 1.10 <sub>1.10-1.10</sub> | -0.05 -4% ~ |
| peek_handover ms (outside frames) | default | 2.65 <sub>2.40-2.80</sub> |  | 3.10 <sub>3.00-3.20</sub> |  |
| peek_handover ms (outside frames) | keeppeeks | 2.70 <sub>2.50-3.00</sub> | +0.05 +2% ~ | 3.15 <sub>3.00-3.40</sub> | +0.05 +2% ~ |
| peek_handover ms (outside frames) | asynchandback | 2.60 <sub>2.40-2.70</sub> | -0.05 -2% ~ | 3.00 <sub>3.00-3.10</sub> | -0.10 -3% ~ |
| peek_handover ms (outside frames) | asynclanding | 2.60 <sub>2.40-2.90</sub> | -0.05 -2% ~ | 3.10 <sub>2.90-3.20</sub> | +0.00 +0% ~ |
| peek_handover ms (outside frames) | earlylanding | 2.45 <sub>2.30-2.60</sub> | -0.20 -8% ~ | 3.05 <sub>2.90-3.30</sub> | -0.05 -2% ~ |
| peek_handover ms (outside frames) | earlylanding+asynchandback+keeppeeks | 2.60 <sub>2.50-2.70</sub> | -0.05 -2% ~ | 3.10 <sub>3.10-3.20</sub> | +0.00 +0% ~ |
| peek_raise ms (outside frames) | default | 0.40 <sub>0.40-0.40</sub> |  | 0.40 <sub>0.30-0.40</sub> |  |
| peek_raise ms (outside frames) | keeppeeks | 0.45 <sub>0.30-0.50</sub> | +0.05 +12% ~ | 0.40 <sub>0.40-0.50</sub> | +0.00 +0% ~ |
| peek_raise ms (outside frames) | asynchandback | 0.40 <sub>0.40-0.40</sub> | +0.00 +0% ~ | 0.40 <sub>0.40-0.50</sub> | +0.00 +0% ~ |
| peek_raise ms (outside frames) | asynclanding | 0.40 <sub>0.30-0.40</sub> | +0.00 +0% ~ | 0.40 <sub>0.30-0.50</sub> | +0.00 +0% ~ |
| peek_raise ms (outside frames) | earlylanding | 0.40 <sub>0.40-0.40</sub> | +0.00 +0% ~ | 0.40 <sub>0.30-0.50</sub> | +0.00 +0% ~ |
| peek_raise ms (outside frames) | earlylanding+asynchandback+keeppeeks | 0.40 <sub>0.30-0.40</sub> | +0.00 +0% ~ | 0.40 <sub>0.40-0.40</sub> | +0.00 +0% ~ |
| cloak ms (outside frames) | default | 7.45 <sub>7.20-7.80</sub> |  | 7.35 <sub>7.10-7.70</sub> |  |
| cloak ms (outside frames) | keeppeeks | 7.80 <sub>7.80-10.40</sub> | +0.35 +5% ~ | 7.45 <sub>7.30-7.70</sub> | +0.10 +1% ~ |
| cloak ms (outside frames) | asynchandback | 7.70 <sub>7.50-8.20</sub> | +0.25 +3% ~ | 7.20 <sub>7.00-7.80</sub> | -0.15 -2% ~ |
| cloak ms (outside frames) | asynclanding | 7.15 <sub>7.00-7.80</sub> | -0.30 -4% ~ | 7.30 <sub>5.80-7.90</sub> | -0.05 -1% ~ |
| cloak ms (outside frames) | earlylanding | 7.60 <sub>6.90-8.10</sub> | +0.15 +2% ~ | 7.25 <sub>7.10-7.80</sub> | -0.10 -1% ~ |
| cloak ms (outside frames) | earlylanding+asynchandback+keeppeeks | 7.50 <sub>7.20-7.70</sub> | +0.05 +1% ~ | 7.45 <sub>6.60-7.80</sub> | +0.10 +1% ~ |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| session_overlays ms (outside frames) | keeppeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | asynchandback | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | asynclanding | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | earlylanding | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | earlylanding+asynchandback+keeppeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |

### resize: vs `default`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | default | 101.41 <sub>89.89-102.71</sub> |  | 98.55 <sub>96.14-100.48</sub> |  |
| input->first frame ms | keeppeeks | 90.50 <sub>84.86-94.77</sub> | -10.91 -11% ~ | 96.30 <sub>94.07-101.26</sub> | -2.25 -2% ~ |
| input->first frame ms | asynchandback | 95.49 <sub>88.17-106.35</sub> | -5.92 -6% ~ | 98.06 <sub>97.43-98.36</sub> | -0.48 -0% ~ |
| input->first frame ms | asynclanding | 94.88 <sub>90.77-101.26</sub> | -6.53 -6% ~ | 99.69 <sub>95.51-101.89</sub> | +1.14 +1% ~ |
| input->first frame ms | earlylanding | 93.59 <sub>86.42-96.89</sub> | -7.82 -8% ~ | 100.50 <sub>93.03-102.26</sub> | +1.95 +2% ~ |
| input->first frame ms | earlylanding+asynchandback+keeppeeks | 91.04 <sub>86.96-94.77</sub> | -10.37 -10% ~ | 98.32 <sub>95.42-100.05</sub> | -0.23 -0% ~ |
| command p50 ms | default | 71.72 <sub>63.40-75.77</sub> |  | 68.26 <sub>67.52-73.75</sub> |  |
| command p50 ms | keeppeeks | 64.01 <sub>59.16-68.16</sub> | -7.71 -11% ~ | 65.78 <sub>64.73-69.55</sub> | -2.48 -4% ~ |
| command p50 ms | asynchandback | 67.56 <sub>63.02-78.75</sub> | -4.17 -6% ~ | 68.03 <sub>67.29-68.34</sub> | -0.23 -0% ~ |
| command p50 ms | asynclanding | 67.66 <sub>62.50-73.66</sub> | -4.06 -6% ~ | 68.61 <sub>67.24-74.34</sub> | +0.35 +1% ~ |
| command p50 ms | earlylanding | 64.61 <sub>61.41-67.59</sub> | -7.11 -10% ~ | 67.91 <sub>67.20-68.71</sub> | -0.35 -1% ~ |
| command p50 ms | earlylanding+asynchandback+keeppeeks | 63.47 <sub>59.48-67.71</sub> | -8.26 -12% ~ | 67.52 <sub>65.80-68.73</sub> | -0.74 -1% ~ |
| command ms (WM side) | default | 71.35 <sub>63.00-75.40</sub> |  | 67.95 <sub>67.20-73.50</sub> |  |
| command ms (WM side) | keeppeeks | 63.65 <sub>58.90-67.80</sub> | -7.70 -11% ~ | 65.50 <sub>64.30-69.20</sub> | -2.45 -4% ~ |
| command ms (WM side) | asynchandback | 67.20 <sub>62.70-78.40</sub> | -4.15 -6% ~ | 67.75 <sub>66.80-68.00</sub> | -0.20 -0% ~ |
| command ms (WM side) | asynclanding | 67.35 <sub>62.20-73.30</sub> | -4.00 -6% ~ | 68.30 <sub>67.00-73.40</sub> | +0.35 +1% ~ |
| command ms (WM side) | earlylanding | 64.25 <sub>61.20-67.10</sub> | -7.10 -10% ~ | 67.45 <sub>66.80-68.40</sub> | -0.50 -1% ~ |
| command ms (WM side) | earlylanding+asynchandback+keeppeeks | 63.05 <sub>59.10-67.40</sub> | -8.30 -12% ~ | 67.30 <sub>65.50-68.50</sub> | -0.65 -1% ~ |
| frame p90 ms | default | 3.38 <sub>3.10-3.57</sub> |  | 4.00 <sub>3.85-4.18</sub> |  |
| frame p90 ms | keeppeeks | 3.60 <sub>3.38-3.73</sub> | +0.23 +7% ~ | 3.85 <sub>3.68-4.10</sub> | -0.15 -4% ~ |
| frame p90 ms | asynchandback | 3.75 <sub>3.50-4.05</sub> | +0.38 +11% ~ | 4.26 <sub>4.11-4.33</sub> | +0.26 +6% ~ |
| frame p90 ms | asynclanding | 4.09 <sub>3.81-5.57</sub> | +0.72 +21% **sep** | 4.77 <sub>4.19-5.03</sub> | +0.77 +19% **sep** |
| frame p90 ms | earlylanding | 5.54 <sub>5.12-9.73</sub> | +2.16 +64% **sep** | 9.97 <sub>6.21-12.91</sub> | +5.97 +149% **sep** |
| frame p90 ms | earlylanding+asynchandback+keeppeeks | 4.63 <sub>4.01-7.43</sub> | +1.26 +37% **sep** | 10.68 <sub>5.34-11.77</sub> | +6.68 +167% **sep** |
| interval p90 ms | default | 5.93 <sub>5.81-6.72</sub> |  | 5.82 <sub>5.79-5.83</sub> |  |
| interval p90 ms | keeppeeks | 5.80 <sub>5.79-5.80</sub> | -0.13 -2% **sep** | 5.80 <sub>5.77-5.81</sub> | -0.02 -0% ~ |
| interval p90 ms | asynchandback | 5.85 <sub>5.78-5.98</sub> | -0.08 -1% ~ | 5.97 <sub>5.81-6.26</sub> | +0.15 +3% ~ |
| interval p90 ms | asynclanding | 6.66 <sub>6.25-7.08</sub> | +0.73 +12% ~ | 7.36 <sub>6.39-7.75</sub> | +1.54 +26% **sep** |
| interval p90 ms | earlylanding | 8.64 <sub>6.15-11.47</sub> | +2.72 +46% ~ | 11.50 <sub>8.18-14.16</sub> | +5.68 +98% **sep** |
| interval p90 ms | earlylanding+asynchandback+keeppeeks | 8.02 <sub>5.86-9.65</sub> | +2.09 +35% ~ | 12.09 <sub>6.16-12.53</sub> | +6.27 +108% **sep** |
| interval p99 ms | default | 45.65 <sub>41.20-47.04</sub> |  | 46.36 <sub>43.61-48.94</sub> |  |
| interval p99 ms | keeppeeks | 49.22 <sub>46.33-56.28</sub> | +3.57 +8% ~ | 59.95 <sub>50.38-66.35</sub> | +13.59 +29% **sep** |
| interval p99 ms | asynchandback | 37.97 <sub>35.88-41.51</sub> | -7.68 -17% ~ | 41.73 <sub>38.36-45.33</sub> | -4.63 -10% ~ |
| interval p99 ms | asynclanding | 37.62 <sub>31.48-41.50</sub> | -8.03 -18% ~ | 40.39 <sub>37.90-42.19</sub> | -5.97 -13% **sep** |
| interval p99 ms | earlylanding | 30.84 <sub>28.97-33.12</sub> | -14.82 -32% **sep** | 31.93 <sub>29.11-33.32</sub> | -14.43 -31% **sep** |
| interval p99 ms | earlylanding+asynchandback+keeppeeks | 30.37 <sub>27.96-32.13</sub> | -15.29 -33% **sep** | 32.28 <sub>31.40-33.24</sub> | -14.08 -30% **sep** |
| interval max ms (median burst) | default | 45.65 <sub>41.20-47.04</sub> |  | 46.36 <sub>43.61-48.94</sub> |  |
| interval max ms (median burst) | keeppeeks | 49.22 <sub>46.33-56.28</sub> | +3.57 +8% ~ | 59.95 <sub>50.38-66.35</sub> | +13.59 +29% **sep** |
| interval max ms (median burst) | asynchandback | 37.97 <sub>35.88-41.51</sub> | -7.68 -17% ~ | 41.73 <sub>38.36-45.33</sub> | -4.63 -10% ~ |
| interval max ms (median burst) | asynclanding | 37.62 <sub>31.48-41.50</sub> | -8.03 -18% ~ | 40.39 <sub>37.90-42.19</sub> | -5.97 -13% **sep** |
| interval max ms (median burst) | earlylanding | 30.84 <sub>28.97-33.12</sub> | -14.82 -32% **sep** | 31.93 <sub>29.11-33.32</sub> | -14.43 -31% **sep** |
| interval max ms (median burst) | earlylanding+asynchandback+keeppeeks | 30.37 <sub>27.96-32.13</sub> | -15.29 -33% **sep** | 32.28 <sub>31.40-33.24</sub> | -14.08 -30% **sep** |
| frames | default | 60.50 <sub>60.00-61.00</sub> |  | 59.50 <sub>59.00-60.00</sub> |  |
| frames | keeppeeks | 60.00 <sub>60.00-60.00</sub> | -0.50 -1% ~ | 60.00 <sub>59.00-60.00</sub> | +0.50 +1% ~ |
| frames | asynchandback | 60.00 <sub>59.00-61.00</sub> | -0.50 -1% ~ | 59.00 <sub>59.00-59.00</sub> | -0.50 -1% ~ |
| frames | asynclanding | 60.50 <sub>60.00-62.00</sub> | +0.00 +0% ~ | 61.00 <sub>60.00-61.00</sub> | +1.50 +3% ~ |
| frames | earlylanding | 59.00 <sub>59.00-59.00</sub> | -1.50 -2% **sep** | 58.00 <sub>57.00-59.00</sub> | -1.50 -3% ~ |
| frames | earlylanding+asynchandback+keeppeeks | 59.00 <sub>59.00-59.00</sub> | -1.50 -2% **sep** | 58.00 <sub>58.00-58.00</sub> | -1.50 -3% **sep** |
| peek_new ms (outside frames) | default | 17.70 <sub>17.50-18.00</sub> |  | 19.80 <sub>18.80-20.10</sub> |  |
| peek_new ms (outside frames) | keeppeeks | 15.40 <sub>15.10-15.50</sub> | -2.30 -13% **sep** | 18.35 <sub>17.60-18.60</sub> | -1.45 -7% **sep** |
| peek_new ms (outside frames) | asynchandback | 17.45 <sub>16.90-18.40</sub> | -0.25 -1% ~ | 20.25 <sub>18.90-20.50</sub> | +0.45 +2% ~ |
| peek_new ms (outside frames) | asynclanding | 17.55 <sub>17.20-18.50</sub> | -0.15 -1% ~ | 20.00 <sub>19.20-20.20</sub> | +0.20 +1% ~ |
| peek_new ms (outside frames) | earlylanding | 17.15 <sub>16.20-17.30</sub> | -0.55 -3% **sep** | 19.85 <sub>19.30-20.10</sub> | +0.05 +0% ~ |
| peek_new ms (outside frames) | earlylanding+asynchandback+keeppeeks | 14.85 <sub>14.00-15.40</sub> | -2.85 -16% **sep** | 17.45 <sub>16.70-18.10</sub> | -2.35 -12% **sep** |
| peek_cover ms (outside frames) | default | 10.60 <sub>10.20-10.70</sub> |  | 11.20 <sub>11.20-11.30</sub> |  |
| peek_cover ms (outside frames) | keeppeeks | 10.65 <sub>10.30-11.20</sub> | +0.05 +0% ~ | 11.60 <sub>11.20-11.80</sub> | +0.40 +4% ~ |
| peek_cover ms (outside frames) | asynchandback | 10.80 <sub>10.50-11.10</sub> | +0.20 +2% ~ | 11.80 <sub>11.70-11.90</sub> | +0.60 +5% **sep** |
| peek_cover ms (outside frames) | asynclanding | 10.80 <sub>10.50-11.20</sub> | +0.20 +2% ~ | 11.40 <sub>11.20-11.70</sub> | +0.20 +2% ~ |
| peek_cover ms (outside frames) | earlylanding | 10.60 <sub>9.90-11.10</sub> | +0.00 +0% ~ | 11.10 <sub>10.90-11.40</sub> | -0.10 -1% ~ |
| peek_cover ms (outside frames) | earlylanding+asynchandback+keeppeeks | 10.35 <sub>9.80-11.00</sub> | -0.25 -2% ~ | 11.00 <sub>10.90-11.20</sub> | -0.20 -2% ~ |
| peek_handover ms (outside frames) | default | 12.80 <sub>10.70-14.00</sub> |  | 13.90 <sub>12.30-15.70</sub> |  |
| peek_handover ms (outside frames) | keeppeeks | 14.20 <sub>11.70-15.20</sub> | +1.40 +11% ~ | 14.35 <sub>11.40-14.60</sub> | +0.45 +3% ~ |
| peek_handover ms (outside frames) | asynchandback | 13.85 <sub>12.50-16.40</sub> | +1.05 +8% ~ | 13.65 <sub>11.20-14.00</sub> | -0.25 -2% ~ |
| peek_handover ms (outside frames) | asynclanding | 12.90 <sub>10.80-14.30</sub> | +0.10 +1% ~ | 13.85 <sub>11.50-15.40</sub> | -0.05 -0% ~ |
| peek_handover ms (outside frames) | earlylanding | 14.15 <sub>11.30-14.40</sub> | +1.35 +11% ~ | 14.00 <sub>12.10-17.40</sub> | +0.10 +1% ~ |
| peek_handover ms (outside frames) | earlylanding+asynchandback+keeppeeks | 12.25 <sub>10.70-14.40</sub> | -0.55 -4% ~ | 12.95 <sub>12.90-14.80</sub> | -0.95 -7% ~ |
| peek_raise ms (outside frames) | default | 1.65 <sub>1.50-1.80</sub> |  | 1.85 <sub>1.70-2.00</sub> |  |
| peek_raise ms (outside frames) | keeppeeks | 1.75 <sub>1.20-2.50</sub> | +0.10 +6% ~ | 1.80 <sub>1.80-2.40</sub> | -0.05 -3% ~ |
| peek_raise ms (outside frames) | asynchandback | 2.20 <sub>1.70-2.50</sub> | +0.55 +33% ~ | 1.85 <sub>1.60-2.30</sub> | +0.00 +0% ~ |
| peek_raise ms (outside frames) | asynclanding | 2.15 <sub>1.20-2.50</sub> | +0.50 +30% ~ | 1.85 <sub>1.30-2.90</sub> | -0.00 -0% ~ |
| peek_raise ms (outside frames) | earlylanding | 2.10 <sub>1.70-2.70</sub> | +0.45 +27% ~ | 2.15 <sub>1.80-3.10</sub> | +0.30 +16% ~ |
| peek_raise ms (outside frames) | earlylanding+asynchandback+keeppeeks | 2.20 <sub>1.80-2.90</sub> | +0.55 +33% ~ | 2.15 <sub>1.80-2.50</sub> | +0.30 +16% ~ |
| cloak ms (outside frames) | default | 20.90 <sub>18.80-24.70</sub> |  | 19.55 <sub>18.10-20.20</sub> |  |
| cloak ms (outside frames) | keeppeeks | 15.35 <sub>14.70-16.80</sub> | -5.55 -27% **sep** | 17.70 <sub>16.10-17.80</sub> | -1.85 -9% **sep** |
| cloak ms (outside frames) | asynchandback | 18.90 <sub>14.20-23.60</sub> | -2.00 -10% ~ | 19.70 <sub>18.80-20.80</sub> | +0.15 +1% ~ |
| cloak ms (outside frames) | asynclanding | 18.45 <sub>17.10-24.10</sub> | -2.45 -12% ~ | 17.80 <sub>17.10-19.00</sub> | -1.75 -9% ~ |
| cloak ms (outside frames) | earlylanding | 16.65 <sub>13.10-19.10</sub> | -4.25 -20% ~ | 18.00 <sub>15.80-18.10</sub> | -1.55 -8% ~ |
| cloak ms (outside frames) | earlylanding+asynchandback+keeppeeks | 16.55 <sub>15.00-19.90</sub> | -4.35 -21% ~ | 18.00 <sub>17.70-22.10</sub> | -1.55 -8% ~ |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| session_overlays ms (outside frames) | keeppeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | asynchandback | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | asynclanding | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | earlylanding | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | earlylanding+asynchandback+keeppeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |

### move: vs `nopeeks`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | nopeeks | 15.45 <sub>14.19-16.55</sub> |  | 16.24 <sub>15.53-17.82</sub> |  |
| input->first frame ms | nopeeks+keepoverlays | 12.66 <sub>12.47-13.36</sub> | -2.79 -18% **sep** | 14.09 <sub>13.46-14.70</sub> | -2.16 -13% **sep** |
| input->first frame ms | default | 19.89 <sub>19.70-20.14</sub> | +4.44 +29% **sep** | 21.48 <sub>21.27-21.86</sub> | +5.23 +32% **sep** |
| command p50 ms | nopeeks | 14.81 <sub>13.72-15.73</sub> |  | 15.29 <sub>14.79-17.19</sub> |  |
| command p50 ms | nopeeks+keepoverlays | 12.11 <sub>11.95-12.19</sub> | -2.71 -18% **sep** | 13.13 <sub>12.44-13.52</sub> | -2.16 -14% **sep** |
| command p50 ms | default | 19.24 <sub>19.01-19.65</sub> | +4.43 +30% **sep** | 20.46 <sub>20.28-20.98</sub> | +5.17 +34% **sep** |
| command ms (WM side) | nopeeks | 14.50 <sub>13.40-15.40</sub> |  | 15.00 <sub>14.50-16.90</sub> |  |
| command ms (WM side) | nopeeks+keepoverlays | 11.85 <sub>11.60-11.90</sub> | -2.65 -18% **sep** | 12.85 <sub>12.20-13.20</sub> | -2.15 -14% **sep** |
| command ms (WM side) | default | 18.95 <sub>18.70-19.30</sub> | +4.45 +31% **sep** | 20.15 <sub>20.00-20.70</sub> | +5.15 +34% **sep** |
| frame p90 ms | nopeeks | 1.41 <sub>1.29-1.43</sub> |  | 1.36 <sub>1.34-1.37</sub> |  |
| frame p90 ms | nopeeks+keepoverlays | 1.42 <sub>1.31-1.43</sub> | +0.01 +0% ~ | 1.38 <sub>1.37-1.38</sub> | +0.02 +1% ~ |
| frame p90 ms | default | 0.85 <sub>0.60-0.89</sub> | -0.55 -39% **sep** | 0.96 <sub>0.93-0.99</sub> | -0.40 -29% **sep** |
| interval p90 ms | nopeeks | 5.74 <sub>5.74-5.74</sub> |  | 5.75 <sub>5.74-5.76</sub> |  |
| interval p90 ms | nopeeks+keepoverlays | 5.74 <sub>5.74-5.74</sub> | +0.00 +0% ~ | 5.75 <sub>5.74-5.76</sub> | +0.00 +0% ~ |
| interval p90 ms | default | 5.75 <sub>5.74-5.75</sub> | +0.01 +0% ~ | 5.75 <sub>5.74-5.76</sub> | +0.00 +0% ~ |
| interval p99 ms | nopeeks | 7.72 <sub>7.48-7.88</sub> |  | 8.73 <sub>8.16-8.86</sub> |  |
| interval p99 ms | nopeeks+keepoverlays | 7.40 <sub>7.22-7.67</sub> | -0.32 -4% ~ | 8.39 <sub>7.45-8.73</sub> | -0.34 -4% ~ |
| interval p99 ms | default | 11.20 <sub>10.76-11.53</sub> | +3.48 +45% **sep** | 12.31 <sub>12.07-12.57</sub> | +3.58 +41% **sep** |
| interval max ms (median burst) | nopeeks | 7.72 <sub>7.48-7.88</sub> |  | 8.73 <sub>8.16-8.86</sub> |  |
| interval max ms (median burst) | nopeeks+keepoverlays | 7.40 <sub>7.22-7.67</sub> | -0.32 -4% ~ | 8.39 <sub>7.45-8.73</sub> | -0.34 -4% ~ |
| interval max ms (median burst) | default | 11.20 <sub>10.76-11.53</sub> | +3.48 +45% **sep** | 12.31 <sub>12.07-12.57</sub> | +3.58 +41% **sep** |
| frames | nopeeks | 62.00 <sub>62.00-62.00</sub> |  | 62.00 <sub>62.00-62.00</sub> |  |
| frames | nopeeks+keepoverlays | 62.00 <sub>62.00-63.00</sub> | +0.00 +0% ~ | 62.00 <sub>62.00-62.00</sub> | +0.00 +0% ~ |
| frames | default | 63.00 <sub>63.00-63.00</sub> | +1.00 +2% **sep** | 63.00 <sub>63.00-63.00</sub> | +1.00 +2% **sep** |
| peek_new ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_new ms (outside frames) | nopeeks+keepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_new ms (outside frames) | default | 5.30 <sub>5.20-5.40</sub> | +5.30 **sep** | 6.05 <sub>5.80-6.70</sub> | +6.05 **sep** |
| peek_cover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_cover ms (outside frames) | nopeeks+keepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | default | 1.05 <sub>1.00-1.10</sub> | +1.05 **sep** | 1.15 <sub>1.10-1.20</sub> | +1.15 **sep** |
| peek_handover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_handover ms (outside frames) | nopeeks+keepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | default | 2.65 <sub>2.40-2.80</sub> | +2.65 **sep** | 3.10 <sub>3.00-3.20</sub> | +3.10 **sep** |
| peek_raise ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_raise ms (outside frames) | nopeeks+keepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | default | 0.40 <sub>0.40-0.40</sub> | +0.40 **sep** | 0.40 <sub>0.30-0.40</sub> | +0.40 **sep** |
| cloak ms (outside frames) | nopeeks | 8.95 <sub>7.70-9.90</sub> |  | 8.75 <sub>8.30-9.30</sub> |  |
| cloak ms (outside frames) | nopeeks+keepoverlays | 7.80 <sub>7.60-8.10</sub> | -1.15 -13% ~ | 8.90 <sub>6.70-9.10</sub> | +0.15 +2% ~ |
| cloak ms (outside frames) | default | 7.45 <sub>7.20-7.80</sub> | -1.50 -17% ~ | 7.35 <sub>7.10-7.70</sub> | -1.40 -16% **sep** |
| session_overlays ms (outside frames) | nopeeks | 3.45 <sub>3.30-3.60</sub> |  | 3.80 <sub>3.50-4.10</sub> |  |
| session_overlays ms (outside frames) | nopeeks+keepoverlays | 1.90 <sub>1.80-1.90</sub> | -1.55 -45% **sep** | 1.90 <sub>1.90-2.20</sub> | -1.90 -50% **sep** |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | -3.45 -100% **sep** | 0.00 <sub>0.00-0.00</sub> | -3.80 -100% **sep** |

### resize: vs `nopeeks`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | nopeeks | 47.85 <sub>44.17-48.12</sub> |  | 47.30 <sub>46.59-48.99</sub> |  |
| input->first frame ms | nopeeks+keepoverlays | 37.00 <sub>35.76-38.44</sub> | -10.85 -23% **sep** | 38.02 <sub>36.52-38.41</sub> | -9.28 -20% **sep** |
| input->first frame ms | default | 101.41 <sub>89.89-102.71</sub> | +53.56 +112% **sep** | 98.55 <sub>96.14-100.48</sub> | +51.25 +108% **sep** |
| command p50 ms | nopeeks | 42.36 <sub>39.54-43.81</sub> |  | 42.22 <sub>40.21-43.85</sub> |  |
| command p50 ms | nopeeks+keepoverlays | 29.48 <sub>26.80-32.69</sub> | -12.89 -30% **sep** | 30.71 <sub>30.07-31.13</sub> | -11.50 -27% **sep** |
| command p50 ms | default | 71.72 <sub>63.40-75.77</sub> | +29.36 +69% **sep** | 68.26 <sub>67.52-73.75</sub> | +26.04 +62% **sep** |
| command ms (WM side) | nopeeks | 42.00 <sub>39.20-43.50</sub> |  | 41.80 <sub>39.90-43.50</sub> |  |
| command ms (WM side) | nopeeks+keepoverlays | 29.10 <sub>26.50-32.40</sub> | -12.90 -31% **sep** | 30.40 <sub>29.80-30.80</sub> | -11.40 -27% **sep** |
| command ms (WM side) | default | 71.35 <sub>63.00-75.40</sub> | +29.35 +70% **sep** | 67.95 <sub>67.20-73.50</sub> | +26.15 +63% **sep** |
| frame p90 ms | nopeeks | 4.37 <sub>3.98-4.71</sub> |  | 4.52 <sub>4.14-4.75</sub> |  |
| frame p90 ms | nopeeks+keepoverlays | 4.67 <sub>4.17-5.00</sub> | +0.30 +7% ~ | 4.96 <sub>4.64-5.19</sub> | +0.45 +10% ~ |
| frame p90 ms | default | 3.38 <sub>3.10-3.57</sub> | -0.99 -23% **sep** | 4.00 <sub>3.85-4.18</sub> | -0.52 -12% ~ |
| interval p90 ms | nopeeks | 5.78 <sub>5.77-5.79</sub> |  | 5.78 <sub>5.77-5.80</sub> |  |
| interval p90 ms | nopeeks+keepoverlays | 5.78 <sub>5.77-5.91</sub> | +0.00 +0% ~ | 5.79 <sub>5.78-5.80</sub> | +0.01 +0% ~ |
| interval p90 ms | default | 5.93 <sub>5.81-6.72</sub> | +0.15 +3% **sep** | 5.82 <sub>5.79-5.83</sub> | +0.04 +1% ~ |
| interval p99 ms | nopeeks | 17.60 <sub>16.86-17.79</sub> |  | 18.95 <sub>16.50-20.13</sub> |  |
| interval p99 ms | nopeeks+keepoverlays | 18.70 <sub>16.45-21.14</sub> | +1.10 +6% ~ | 20.60 <sub>19.24-22.65</sub> | +1.65 +9% ~ |
| interval p99 ms | default | 45.65 <sub>41.20-47.04</sub> | +28.05 +159% **sep** | 46.36 <sub>43.61-48.94</sub> | +27.41 +145% **sep** |
| interval max ms (median burst) | nopeeks | 17.60 <sub>16.86-17.79</sub> |  | 18.95 <sub>16.50-20.13</sub> |  |
| interval max ms (median burst) | nopeeks+keepoverlays | 18.70 <sub>16.45-21.14</sub> | +1.10 +6% ~ | 20.60 <sub>19.24-22.65</sub> | +1.65 +9% ~ |
| interval max ms (median burst) | default | 45.65 <sub>41.20-47.04</sub> | +28.05 +159% **sep** | 46.36 <sub>43.61-48.94</sub> | +27.41 +145% **sep** |
| frames | nopeeks | 62.00 <sub>62.00-62.00</sub> |  | 62.00 <sub>62.00-62.00</sub> |  |
| frames | nopeeks+keepoverlays | 62.00 <sub>62.00-62.00</sub> | +0.00 +0% ~ | 62.00 <sub>62.00-62.00</sub> | +0.00 +0% ~ |
| frames | default | 60.50 <sub>60.00-61.00</sub> | -1.50 -2% **sep** | 59.50 <sub>59.00-60.00</sub> | -2.50 -4% **sep** |
| peek_new ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_new ms (outside frames) | nopeeks+keepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_new ms (outside frames) | default | 17.70 <sub>17.50-18.00</sub> | +17.70 **sep** | 19.80 <sub>18.80-20.10</sub> | +19.80 **sep** |
| peek_cover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_cover ms (outside frames) | nopeeks+keepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | default | 10.60 <sub>10.20-10.70</sub> | +10.60 **sep** | 11.20 <sub>11.20-11.30</sub> | +11.20 **sep** |
| peek_handover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_handover ms (outside frames) | nopeeks+keepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | default | 12.80 <sub>10.70-14.00</sub> | +12.80 **sep** | 13.90 <sub>12.30-15.70</sub> | +13.90 **sep** |
| peek_raise ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_raise ms (outside frames) | nopeeks+keepoverlays | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | default | 1.65 <sub>1.50-1.80</sub> | +1.65 **sep** | 1.85 <sub>1.70-2.00</sub> | +1.85 **sep** |
| cloak ms (outside frames) | nopeeks | 21.40 <sub>20.70-23.00</sub> |  | 19.90 <sub>18.50-21.40</sub> |  |
| cloak ms (outside frames) | nopeeks+keepoverlays | 18.30 <sub>15.50-20.50</sub> | -3.10 -14% **sep** | 18.90 <sub>18.40-19.50</sub> | -1.00 -5% ~ |
| cloak ms (outside frames) | default | 20.90 <sub>18.80-24.70</sub> | -0.50 -2% ~ | 19.55 <sub>18.10-20.20</sub> | -0.35 -2% ~ |
| session_overlays ms (outside frames) | nopeeks | 16.00 <sub>14.80-17.30</sub> |  | 17.90 <sub>17.00-20.40</sub> |  |
| session_overlays ms (outside frames) | nopeeks+keepoverlays | 6.85 <sub>5.80-7.30</sub> | -9.15 -57% **sep** | 7.10 <sub>6.20-8.10</sub> | -10.80 -60% **sep** |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | -16.00 -100% **sep** | 0.00 <sub>0.00-0.00</sub> | -17.90 -100% **sep** |
