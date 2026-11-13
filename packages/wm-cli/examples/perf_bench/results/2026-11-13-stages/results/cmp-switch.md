
### switch: vs `default`

| metric | mode | load 0 | change | CPU 8 | change |
| --- | --- | ---: | --- | ---: | --- |
| input->first frame ms | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| input->first frame ms | noslidepeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| input->first frame ms | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| command p50 ms | default | 27.66 <sub>26.70-28.46</sub> |  | 34.42 <sub>31.60-35.56</sub> |  |
| command p50 ms | noslidepeeks | 27.87 <sub>25.65-28.74</sub> | +0.21 +1% ~ | 35.11 <sub>33.07-36.08</sub> | +0.69 +2% ~ |
| command p50 ms | nopeeks | 27.56 <sub>26.36-28.41</sub> | -0.09 -0% ~ | 34.79 <sub>32.71-38.64</sub> | +0.37 +1% ~ |
| command ms (WM side) | default | 27.35 <sub>26.40-28.20</sub> |  | 34.10 <sub>31.30-35.30</sub> |  |
| command ms (WM side) | noslidepeeks | 27.55 <sub>25.40-28.40</sub> | +0.20 +1% ~ | 34.80 <sub>32.80-35.80</sub> | +0.70 +2% ~ |
| command ms (WM side) | nopeeks | 27.25 <sub>26.00-28.00</sub> | -0.10 -0% ~ | 34.45 <sub>32.40-37.30</sub> | +0.35 +1% ~ |
| frame p90 ms | default | 1.23 <sub>1.15-1.36</sub> |  | 1.46 <sub>1.31-1.58</sub> |  |
| frame p90 ms | noslidepeeks | 1.31 <sub>1.19-1.33</sub> | +0.08 +6% ~ | 1.40 <sub>1.37-1.50</sub> | -0.06 -4% ~ |
| frame p90 ms | nopeeks | 1.25 <sub>1.19-1.38</sub> | +0.02 +2% ~ | 1.57 <sub>1.32-1.72</sub> | +0.10 +7% ~ |
| interval p90 ms | default | 5.75 <sub>5.75-5.76</sub> |  | 5.77 <sub>5.77-5.78</sub> |  |
| interval p90 ms | noslidepeeks | 5.76 <sub>5.75-5.76</sub> | +0.00 +0% ~ | 5.77 <sub>5.76-5.78</sub> | +0.00 +0% ~ |
| interval p90 ms | nopeeks | 5.75 <sub>5.75-5.76</sub> | +0.00 +0% ~ | 5.77 <sub>5.77-5.78</sub> | +0.00 +0% ~ |
| interval p99 ms | default | 16.23 <sub>14.91-18.47</sub> |  | 23.29 <sub>22.89-24.22</sub> |  |
| interval p99 ms | noslidepeeks | 16.42 <sub>15.26-16.83</sub> | +0.19 +1% ~ | 22.87 <sub>22.81-23.92</sub> | -0.42 -2% ~ |
| interval p99 ms | nopeeks | 17.63 <sub>15.50-17.90</sub> | +1.40 +9% ~ | 22.57 <sub>21.99-24.81</sub> | -0.71 -3% ~ |
| interval max ms (median burst) | default | 16.23 <sub>14.91-18.47</sub> |  | 23.29 <sub>22.89-24.22</sub> |  |
| interval max ms (median burst) | noslidepeeks | 16.42 <sub>15.26-16.83</sub> | +0.19 +1% ~ | 23.40 <sub>22.81-24.74</sub> | +0.11 +0% ~ |
| interval max ms (median burst) | nopeeks | 17.63 <sub>15.50-17.90</sub> | +1.40 +9% ~ | 23.66 <sub>22.71-24.81</sub> | +0.37 +2% ~ |
| frames | default | 98.00 <sub>98.00-98.00</sub> |  | 99.00 <sub>98.00-99.00</sub> |  |
| frames | noslidepeeks | 98.00 <sub>98.00-98.00</sub> | +0.00 +0% ~ | 98.50 <sub>98.00-100.00</sub> | -0.50 -1% ~ |
| frames | nopeeks | 98.00 <sub>98.00-98.00</sub> | +0.00 +0% ~ | 99.00 <sub>98.00-99.00</sub> | +0.00 +0% ~ |
| peek_new ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_new ms (outside frames) | noslidepeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_new ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_cover ms (outside frames) | noslidepeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_cover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_handover ms (outside frames) | noslidepeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_handover ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| peek_raise ms (outside frames) | noslidepeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| peek_raise ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| cloak ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| cloak ms (outside frames) | noslidepeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| cloak ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | default | 0.00 <sub>0.00-0.00</sub> |  | 0.00 <sub>0.00-0.00</sub> |  |
| session_overlays ms (outside frames) | noslidepeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
| session_overlays ms (outside frames) | nopeeks | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ | 0.00 <sub>0.00-0.00</sub> | +0.00 ~ |
