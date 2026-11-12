## t

Median over rounds (up to 4), round range in small print. Lower is better, except interval p50, which is the refresh period.

### move

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> |
| interval p90 ms | 5.75 <sub>5.74–5.75</sub> | 5.75 <sub>5.75–5.76</sub> | 5.76 <sub>5.75–5.77</sub> | 5.76 <sub>5.76–5.77</sub> |
| frame p90 ms | 0.95 <sub>0.94–1.06</sub> | 0.95 <sub>0.89–0.96</sub> | 0.66 <sub>0.51–0.76</sub> | 0.56 <sub>0.55–0.61</sub> |
| frames >2x budget | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> | 2.00 <sub>1.00–2.00</sub> | 2.00 <sub>2.00–2.00</sub> |
| tick ms/frame | 0.66 <sub>0.59–0.69</sub> | 0.56 <sub>0.54–0.56</sub> | 0.82 <sub>0.79–0.86</sub> | 0.84 <sub>0.82–0.86</sub> |
| peek_frame ms/frame | n/a | n/a | 0.44 <sub>0.43–0.44</sub> | 0.42 <sub>0.41–0.47</sub> |
| input->first frame ms | 16.48 <sub>15.20–16.92</sub> | 18.99 <sub>18.47–19.40</sub> | n/a | n/a |
| command p50 ms | 15.68 <sub>14.64–16.17</sub> | 18.36 <sub>17.99–18.80</sub> | 24.45 <sub>23.73–25.08</sub> | 24.25 <sub>23.83–25.15</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-31%) | better (-31%) | better (-42%) | better (-41%) | ≈ | ≈ |
| frames >2x budget | worse (2.00 vs 0) | worse (2.00 vs 0) | worse (2.00 vs 0) | worse (2.00 vs 0) | ≈ | ≈ |
| tick ms/frame | worse (+25%) | worse (+49%) | worse (+27%) | worse (+51%) | ≈ | better (-16%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | worse (+15%) |
| command p50 ms | worse (+56%) | worse (+33%) | worse (+55%) | worse (+32%) | ≈ | worse (+17%) |

### float

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 8.41 <sub>6.51–10.83</sub> | 8.36 <sub>6.12–9.60</sub> | 9.19 <sub>8.47–9.54</sub> | 6.79 <sub>6.51–7.30</sub> |
| frame p90 ms | 7.11 <sub>6.40–9.83</sub> | 8.00 <sub>4.25–9.50</sub> | 8.53 <sub>7.40–9.25</sub> | 3.98 <sub>3.70–4.43</sub> |
| frames >2x budget | 5.50 <sub>4.00–6.00</sub> | 4.00 <sub>4.00–4.00</sub> | 4.00 <sub>4.00–4.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 3.74 <sub>3.52–3.86</sub> | 3.62 <sub>3.60–3.90</sub> | 4.03 <sub>3.97–4.21</sub> | 5.04 <sub>4.97–5.11</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 3.13 <sub>3.08–3.20</sub> |
| input->first frame ms | 88.37 <sub>74.08–106.79</sub> | 110.36 <sub>84.27–114.45</sub> | n/a | n/a |
| command p50 ms | 74.41 <sub>66.83–84.73</sub> | 105.70 <sub>102.04–107.21</sub> | 99.54 <sub>97.55–104.92</sub> | 137.87 <sub>130.28–140.15</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | better (-26%) | ≈ |
| frame p90 ms | ≈ | ≈ | better (-44%) | ≈ | better (-53%) | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | worse (+25%) | worse (+25%) | ≈ |
| tick ms/frame | worse (+8%) | worse (+11%) | worse (+35%) | worse (+39%) | worse (+25%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+34%) | ≈ | worse (+85%) | worse (+30%) | worse (+39%) | worse (+42%) |

### minimize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.79 <sub>5.77–5.79</sub> | 6.00 <sub>5.79–6.17</sub> | 5.78 <sub>5.77–5.78</sub> | 5.78 <sub>5.77–5.78</sub> |
| frame p90 ms | 3.50 <sub>3.10–4.33</sub> | 5.43 <sub>5.06–6.00</sub> | 3.63 <sub>3.34–3.87</sub> | 3.58 <sub>3.35–3.74</sub> |
| frames >2x budget | 5.00 <sub>5.00–7.00</sub> | 4.50 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> | 6.00 <sub>6.00–6.00</sub> |
| tick ms/frame | 1.77 <sub>1.56–1.87</sub> | 1.56 <sub>1.47–1.60</sub> | 1.77 <sub>1.74–1.80</sub> | 1.77 <sub>1.76–1.78</sub> |
| peek_frame ms/frame | n/a | n/a | 0.70 <sub>0.00–0.89</sub> | 0.86 <sub>0.00–1.19</sub> |
| input->first frame ms | 101.12 <sub>97.46–117.15</sub> | 122.73 <sub>109.65–130.30</sub> | n/a | n/a |
| command p50 ms | 47.78 <sub>28.04–49.52</sub> | 57.50 <sub>36.34–59.19</sub> | 67.97 <sub>33.94–69.52</sub> | 64.65 <sub>32.72–68.52</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | ≈ | better (-33%) | ≈ | better (-34%) | ≈ | worse (+55%) |
| frames >2x budget | ≈ | ≈ | ≈ | worse (+33%) | worse (+20%) | ≈ |
| tick ms/frame | ≈ | worse (+13%) | ≈ | worse (+13%) | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |

### resize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.72–5.72</sub> |
| interval p90 ms | 9.16 <sub>9.01–10.21</sub> | 8.03 <sub>6.69–9.18</sub> | 6.55 <sub>5.97–6.96</sub> | 6.82 <sub>5.85–6.96</sub> |
| frame p90 ms | 8.67 <sub>6.41–10.15</sub> | 7.67 <sub>6.53–8.69</sub> | 4.22 <sub>4.00–4.47</sub> | 4.30 <sub>4.27–4.37</sub> |
| frames >2x budget | 5.00 <sub>4.00–5.00</sub> | 4.00 <sub>4.00–4.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.35 <sub>3.98–4.59</sub> | 3.77 <sub>3.27–3.97</sub> | 5.42 <sub>4.75–5.56</sub> | 5.10 <sub>4.90–5.48</sub> |
| peek_frame ms/frame | n/a | n/a | 3.38 <sub>3.16–3.46</sub> | 3.29 <sub>3.25–3.40</sub> |
| input->first frame ms | 88.55 <sub>81.09–98.42</sub> | 101.81 <sub>94.27–114.85</sub> | n/a | n/a |
| command p50 ms | 76.21 <sub>70.57–78.32</sub> | 90.07 <sub>87.26–94.09</sub> | 135.34 <sub>133.14–138.25</sub> | 130.36 <sub>125.26–133.10</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | better (-28%) | ≈ | better (-26%) | ≈ | ≈ | ≈ |
| frame p90 ms | better (-51%) | better (-45%) | better (-50%) | better (-44%) | ≈ | ≈ |
| frames >2x budget | ≈ | worse (+25%) | ≈ | worse (+25%) | ≈ | ≈ |
| tick ms/frame | worse (+25%) | worse (+44%) | worse (+17%) | worse (+35%) | ≈ | better (-13%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+78%) | worse (+50%) | worse (+71%) | worse (+45%) | ≈ | worse (+18%) |

### switch

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.71</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.76 <sub>5.76–5.77</sub> | 5.76 <sub>5.76–5.77</sub> | 5.77 <sub>5.77–5.77</sub> | 5.77 <sub>5.77–5.77</sub> |
| frame p90 ms | 3.65 <sub>3.50–3.76</sub> | 1.73 <sub>1.63–1.77</sub> | 1.79 <sub>1.66–1.89</sub> | 1.76 <sub>1.74–1.84</sub> |
| frames >2x budget | 2.00 <sub>2.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> |
| tick ms/frame | 2.40 <sub>2.34–2.48</sub> | 1.75 <sub>1.70–1.84</sub> | 1.91 <sub>1.86–1.97</sub> | 1.93 <sub>1.92–1.97</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> |
| input->first frame ms | n/a | n/a | n/a | n/a |
| command p50 ms | 36.68 <sub>31.07–37.93</sub> | 64.20 <sub>53.65–66.29</sub> | 54.34 <sub>52.23–56.00</sub> | 54.36 <sub>52.98–60.92</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-51%) | ≈ | better (-52%) | ≈ | ≈ | better (-53%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | better (-20%) | worse (+9%) | better (-20%) | worse (+10%) | ≈ | better (-27%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+48%) | ≈ | worse (+48%) | ≈ | ≈ | worse (+75%) |
