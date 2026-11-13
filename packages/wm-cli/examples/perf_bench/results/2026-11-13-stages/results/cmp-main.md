
### resize: vs `main`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | main | 36.78 <sub>35.77-37.90</sub> |  | 39.06 <sub>37.17-50.25</sub> |  |
| input->first frame ms | nopeeks | 39.75 <sub>38.63-41.66</sub> | +2.97 +8% **sep** | 41.84 <sub>37.88-42.87</sub> | +2.78 +7% ~ |
| input->first frame ms | default | 87.85 <sub>78.13-100.09</sub> | +51.07 +139% **sep** | 85.42 <sub>84.76-87.98</sub> | +46.36 +119% **sep** |
| command p50 ms | main | 33.25 <sub>32.30-34.58</sub> |  | 33.17 <sub>32.34-44.06</sub> |  |
| command p50 ms | nopeeks | 35.49 <sub>33.03-35.69</sub> | +2.24 +7% ~ | 33.89 <sub>30.17-34.95</sub> | +0.72 +2% ~ |
| command p50 ms | default | 54.98 <sub>51.65-69.01</sub> | +21.73 +65% **sep** | 53.53 <sub>52.34-55.46</sub> | +20.36 +61% **sep** |
| command ms (WM side) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| command ms (WM side) | nopeeks | 35.10 <sub>32.60-35.30</sub> | +35.10 **sep** | 33.55 <sub>30.00-34.60</sub> | +33.55 **sep** |
| command ms (WM side) | default | 54.65 <sub>51.30-68.60</sub> | +54.65 **sep** | 53.15 <sub>52.00-55.10</sub> | +53.15 **sep** |
| frame p90 ms | main | 4.33 <sub>3.91-4.70</sub> |  | 4.62 <sub>4.37-5.97</sub> |  |
| frame p90 ms | nopeeks | 4.70 <sub>4.58-4.91</sub> | +0.36 +8% ~ | 4.26 <sub>3.89-4.62</sub> | -0.36 -8% ~ |
| frame p90 ms | default | 3.25 <sub>3.09-3.52</sub> | -1.08 -25% **sep** | 3.51 <sub>3.39-3.64</sub> | -1.11 -24% **sep** |
| interval p90 ms | main | 5.79 <sub>5.77-5.80</sub> |  | 5.77 <sub>5.77-7.47</sub> |  |
| interval p90 ms | nopeeks | 5.80 <sub>5.79-5.91</sub> | +0.01 +0% ~ | 5.78 <sub>5.76-5.79</sub> | +0.01 +0% ~ |
| interval p90 ms | default | 5.84 <sub>5.79-5.89</sub> | +0.05 +1% ~ | 5.83 <sub>5.78-5.96</sub> | +0.07 +1% ~ |
| interval p99 ms | main | 37.25 <sub>29.71-40.75</sub> |  | 30.07 <sub>26.02-39.28</sub> |  |
| interval p99 ms | nopeeks | 18.52 <sub>17.88-19.99</sub> | -18.73 -50% **sep** | 17.08 <sub>16.23-19.55</sub> | -12.99 -43% **sep** |
| interval p99 ms | default | 44.17 <sub>39.01-50.11</sub> | +6.92 +19% ~ | 36.82 <sub>33.13-38.28</sub> | +6.75 +22% ~ |
| interval max ms (median burst) | main | 37.25 <sub>29.71-40.75</sub> |  | 30.07 <sub>26.02-39.28</sub> |  |
| interval max ms (median burst) | nopeeks | 18.52 <sub>17.88-19.99</sub> | -18.73 -50% **sep** | 17.08 <sub>16.23-19.55</sub> | -12.99 -43% **sep** |
| interval max ms (median burst) | default | 44.17 <sub>39.01-50.11</sub> | +6.92 +19% ~ | 36.82 <sub>33.13-38.28</sub> | +6.75 +22% ~ |
| frames | main | 63.00 <sub>63.00-63.00</sub> |  | 63.00 <sub>62.00-63.00</sub> |  |
| frames | nopeeks | 62.00 <sub>61.00-62.00</sub> | -1.00 -2% **sep** | 62.00 <sub>61.00-62.00</sub> | -1.00 -2% ~ |
| frames | default | 60.00 <sub>59.00-60.00</sub> | -3.00 -5% **sep** | 59.00 <sub>59.00-59.00</sub> | -4.00 -6% **sep** |
| peek_new ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_new ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_new ms (outside frames) | default | 6.45 <sub>6.00-7.30</sub> | +6.45 **sep** | 6.75 <sub>6.60-7.00</sub> | +6.75 **sep** |
| peek_cover ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_cover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | default | 8.65 <sub>8.40-9.60</sub> | +8.65 **sep** | 8.60 <sub>8.10-8.90</sub> | +8.60 **sep** |
| peek_handover ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_handover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | default | 2.55 <sub>2.40-2.90</sub> | +2.55 **sep** | 2.75 <sub>2.70-2.90</sub> | +2.75 **sep** |
| peek_raise ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_raise ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | default | 4.55 <sub>3.90-5.00</sub> | +4.55 **sep** | 5.85 <sub>4.70-6.70</sub> | +5.85 **sep** |
| cloak ms (outside frames) | main | 21.50 <sub>20.90-22.10</sub> |  | 18.75 <sub>18.40-24.90</sub> |  |
| cloak ms (outside frames) | nopeeks | 21.65 <sub>21.20-23.20</sub> | +0.15 +1% ~ | 19.00 <sub>16.70-20.00</sub> | +0.25 +1% ~ |
| cloak ms (outside frames) | default | 27.90 <sub>26.10-35.30</sub> | +6.40 +30% **sep** | 22.65 <sub>19.60-23.80</sub> | +3.90 +21% ~ |
| session_overlays ms (outside frames) | main | 6.85 <sub>6.40-7.40</sub> |  | 7.40 <sub>6.90-10.50</sub> |  |
| session_overlays ms (outside frames) | nopeeks | 7.55 <sub>6.80-8.10</sub> | +0.70 +10% ~ | 8.90 <sub>8.30-9.70</sub> | +1.50 +20% ~ |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | -6.85 -100% **sep** | 0.00 <sub>0.00-0.00</sub> | -7.40 -100% **sep** |

### switch: vs `main`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| input->first frame ms | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| input->first frame ms | default | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| command p50 ms | main | 42.45 <sub>39.92-48.50</sub> |  | 37.50 <sub>35.26-38.35</sub> |  |
| command p50 ms | nopeeks | 31.50 <sub>28.72-33.27</sub> | -10.96 -26% **sep** | 26.52 <sub>24.85-28.11</sub> | -10.98 -29% **sep** |
| command p50 ms | default | 32.67 <sub>29.94-34.79</sub> | -9.78 -23% **sep** | 27.65 <sub>27.20-29.73</sub> | -9.85 -26% **sep** |
| command ms (WM side) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| command ms (WM side) | nopeeks | 31.15 <sub>28.40-32.90</sub> | +31.15 **sep** | 26.20 <sub>24.50-27.80</sub> | +26.20 **sep** |
| command ms (WM side) | default | 32.35 <sub>29.60-34.50</sub> | +32.35 **sep** | 27.35 <sub>26.90-29.40</sub> | +27.35 **sep** |
| frame p90 ms | main | 0.94 <sub>0.82-1.03</sub> |  | 0.76 <sub>0.63-0.84</sub> |  |
| frame p90 ms | nopeeks | 1.25 <sub>1.21-1.30</sub> | +0.32 +34% **sep** | 1.21 <sub>1.16-1.34</sub> | +0.45 +59% **sep** |
| frame p90 ms | default | 1.23 <sub>1.20-1.47</sub> | +0.30 +32% **sep** | 1.32 <sub>1.21-1.48</sub> | +0.56 +74% **sep** |
| interval p90 ms | main | 5.76 <sub>5.76-5.77</sub> |  | 5.75 <sub>5.74-5.76</sub> |  |
| interval p90 ms | nopeeks | 5.76 <sub>5.76-5.78</sub> | +0.00 +0% ~ | 5.75 <sub>5.75-5.75</sub> | +0.00 +0% ~ |
| interval p90 ms | default | 5.78 <sub>5.76-5.78</sub> | +0.02 +0% ~ | 5.75 <sub>5.75-5.76</sub> | +0.00 +0% ~ |
| interval p99 ms | main | 79.09 <sub>78.52-83.11</sub> |  | 71.70 <sub>70.95-78.22</sub> |  |
| interval p99 ms | nopeeks | 20.62 <sub>20.34-20.81</sub> | -58.48 -74% **sep** | 18.06 <sub>17.42-20.01</sub> | -53.64 -75% **sep** |
| interval p99 ms | default | 20.91 <sub>20.59-22.34</sub> | -58.19 -74% **sep** | 20.76 <sub>20.03-21.30</sub> | -50.95 -71% **sep** |
| interval max ms (median burst) | main | 79.09 <sub>78.52-83.11</sub> |  | 71.70 <sub>70.95-78.22</sub> |  |
| interval max ms (median burst) | nopeeks | 20.62 <sub>20.34-20.81</sub> | -58.48 -74% **sep** | 20.36 <sub>20.01-21.07</sub> | -51.34 -72% **sep** |
| interval max ms (median burst) | default | 20.91 <sub>20.59-22.34</sub> | -58.19 -74% **sep** | 20.82 <sub>20.03-21.30</sub> | -50.88 -71% **sep** |
| frames | main | 97.00 <sub>96.00-97.00</sub> |  | 97.00 <sub>97.00-97.00</sub> |  |
| frames | nopeeks | 99.00 <sub>99.00-99.00</sub> | +2.00 +2% **sep** | 99.00 <sub>98.00-99.00</sub> | +2.00 +2% **sep** |
| frames | default | 99.00 <sub>99.00-99.00</sub> | +2.00 +2% **sep** | 99.50 <sub>99.00-100.00</sub> | +2.50 +3% **sep** |
| peek_new ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_new ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_new ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_cover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_handover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_raise ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| cloak ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| cloak ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| cloak ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | main | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| session_overlays ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
