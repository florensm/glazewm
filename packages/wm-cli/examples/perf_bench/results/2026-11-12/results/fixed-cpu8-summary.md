## t

Median over rounds (up to 4), round range in small print. Lower is better, except interval p50, which is the refresh period.

### move

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.76 <sub>5.75–5.77</sub> | 5.76 <sub>5.75–5.76</sub> | 5.75 <sub>5.75–5.76</sub> | 5.76 <sub>5.76–5.77</sub> |
| frame p90 ms | 1.17 <sub>1.12–1.22</sub> | 1.06 <sub>1.01–1.10</sub> | 0.73 <sub>0.63–0.84</sub> | 0.67 <sub>0.64–0.76</sub> |
| frames >2x budget | 1.00 <sub>0.00–1.00</sub> | 1.00 <sub>1.00–1.00</sub> | 1.00 <sub>1.00–2.00</sub> | 1.50 <sub>1.00–2.00</sub> |
| tick ms/frame | 0.77 <sub>0.76–0.82</sub> | 0.63 <sub>0.62–0.69</sub> | 0.96 <sub>0.93–0.98</sub> | 0.95 <sub>0.91–1.01</sub> |
| peek_frame ms/frame | n/a | n/a | 0.41 <sub>0.39–0.45</sub> | 0.45 <sub>0.43–0.46</sub> |
| input->first frame ms | 17.52 <sub>16.94–18.32</sub> | 20.05 <sub>19.87–20.51</sub> | n/a | n/a |
| command p50 ms | 16.55 <sub>16.19–17.57</sub> | 19.36 <sub>19.08–19.70</sub> | 26.08 <sub>24.97–26.61</sub> | 25.45 <sub>25.18–26.17</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-38%) | better (-32%) | better (-43%) | better (-38%) | ≈ | better (-9%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | worse (+25%) | worse (+53%) | worse (+23%) | worse (+51%) | ≈ | better (-18%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | worse (+14%) |
| command p50 ms | worse (+58%) | worse (+35%) | worse (+54%) | worse (+31%) | ≈ | worse (+17%) |

### float

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 10.28 <sub>5.76–10.93</sub> | 9.90 <sub>9.57–10.04</sub> | 8.22 <sub>6.68–10.46</sub> | 7.12 <sub>6.44–7.43</sub> |
| frame p90 ms | 7.84 <sub>4.17–10.23</sub> | 9.80 <sub>9.55–10.02</sub> | 7.38 <sub>4.95–9.86</sub> | 5.02 <sub>4.84–5.10</sub> |
| frames >2x budget | 5.50 <sub>5.00–6.00</sub> | 4.50 <sub>4.00–5.00</sub> | 4.00 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.21 <sub>1.89–4.79</sub> | 4.15 <sub>3.88–4.34</sub> | 4.27 <sub>4.20–4.83</sub> | 5.71 <sub>5.68–6.07</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 3.61 <sub>3.52–3.94</sub> |
| input->first frame ms | 109.20 <sub>77.47–121.14</sub> | 106.39 <sub>91.50–136.61</sub> | n/a | n/a |
| command p50 ms | 91.48 <sub>60.28–97.22</sub> | 99.81 <sub>88.12–120.93</sub> | 112.44 <sub>93.51–121.90</sub> | 163.28 <sub>146.91–165.41</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | better (-28%) | ≈ | ≈ |
| frame p90 ms | ≈ | ≈ | ≈ | better (-49%) | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | ≈ | worse (+35%) | worse (+38%) | worse (+34%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | ≈ | ≈ | worse (+78%) | worse (+64%) | worse (+45%) | ≈ |

### minimize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.85 <sub>5.81–6.22</sub> | 6.50 <sub>6.05–6.80</sub> | 5.79 <sub>5.78–5.79</sub> | 5.79 <sub>5.78–5.79</sub> |
| frame p90 ms | 3.88 <sub>3.26–5.22</sub> | 5.65 <sub>4.92–5.95</sub> | 4.12 <sub>3.92–4.37</sub> | 4.12 <sub>3.83–4.38</sub> |
| frames >2x budget | 7.00 <sub>6.00–7.00</sub> | 6.50 <sub>6.00–7.00</sub> | 7.00 <sub>6.00–7.00</sub> | 7.00 <sub>6.00–7.00</sub> |
| tick ms/frame | 2.10 <sub>1.87–2.14</sub> | 1.73 <sub>1.70–1.85</sub> | 1.98 <sub>1.92–2.17</sub> | 2.06 <sub>2.00–2.14</sub> |
| peek_frame ms/frame | n/a | n/a | 1.29 <sub>0.83–1.36</sub> | 1.23 <sub>0.00–1.35</sub> |
| input->first frame ms | 113.29 <sub>104.28–132.98</sub> | 126.62 <sub>115.41–153.98</sub> | n/a | n/a |
| command p50 ms | 56.28 <sub>35.73–58.93</sub> | 68.95 <sub>40.55–70.74</sub> | 77.41 <sub>73.34–78.85</sub> | 72.66 <sub>70.88–76.09</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | better (-11%) | ≈ | better (-11%) | ≈ | ≈ |
| frame p90 ms | ≈ | better (-27%) | ≈ | better (-27%) | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | worse (+14%) | ≈ | worse (+19%) | ≈ | better (-18%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+38%) | worse (+12%) | worse (+29%) | worse (+5%) | ≈ | ≈ |

### resize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 10.87 <sub>10.61–11.72</sub> | 10.63 <sub>9.70–11.01</sub> | 6.62 <sub>6.36–7.45</sub> | 6.96 <sub>6.54–7.36</sub> |
| frame p90 ms | 10.64 <sub>10.51–11.35</sub> | 10.14 <sub>8.64–10.75</sub> | 5.20 <sub>5.12–5.32</sub> | 5.29 <sub>4.67–5.35</sub> |
| frames >2x budget | 5.00 <sub>4.00–6.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.91 <sub>4.34–5.24</sub> | 4.28 <sub>4.13–4.40</sub> | 6.12 <sub>6.06–6.49</sub> | 6.06 <sub>5.36–6.37</sub> |
| peek_frame ms/frame | n/a | n/a | 3.81 <sub>3.43–3.95</sub> | 3.84 <sub>3.73–3.98</sub> |
| input->first frame ms | 105.49 <sub>83.90–118.32</sub> | 128.50 <sub>103.68–133.35</sub> | n/a | n/a |
| command p50 ms | 84.55 <sub>73.77–87.96</sub> | 114.23 <sub>94.37–115.55</sub> | 159.51 <sub>145.15–161.91</sub> | 161.10 <sub>138.30–163.32</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | better (-39%) | better (-38%) | better (-36%) | better (-35%) | ≈ | ≈ |
| frame p90 ms | better (-51%) | better (-49%) | better (-50%) | better (-48%) | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | worse (+25%) | worse (+43%) | worse (+23%) | worse (+42%) | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | ≈ |
| command p50 ms | worse (+89%) | worse (+40%) | worse (+91%) | worse (+41%) | ≈ | worse (+35%) |

### switch

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.78 <sub>5.77–5.79</sub> | 5.77 <sub>5.77–5.77</sub> | 5.77 <sub>5.76–5.78</sub> | 5.78 <sub>5.77–5.78</sub> |
| frame p90 ms | 4.07 <sub>3.97–4.52</sub> | 2.13 <sub>1.98–2.24</sub> | 2.10 <sub>2.08–2.27</sub> | 2.23 <sub>2.19–2.27</sub> |
| frames >2x budget | 2.50 <sub>2.00–3.00</sub> | 3.00 <sub>3.00–4.00</sub> | 3.00 <sub>3.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> |
| tick ms/frame | 2.77 <sub>2.65–2.91</sub> | 2.01 <sub>1.85–2.19</sub> | 2.10 <sub>1.98–2.14</sub> | 2.15 <sub>2.06–2.21</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> |
| input->first frame ms | n/a | n/a | n/a | n/a |
| command p50 ms | 38.59 <sub>38.27–39.61</sub> | 60.65 <sub>53.32–66.11</sub> | 59.88 <sub>51.50–90.77</sub> | 56.43 <sub>54.22–57.31</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default | nopeeks vs main |
| --- | --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-48%) | ≈ | better (-45%) | ≈ | ≈ | better (-48%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | better (-24%) | ≈ | better (-23%) | ≈ | ≈ | better (-28%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+55%) | ≈ | worse (+46%) | ≈ | ≈ | worse (+57%) |
