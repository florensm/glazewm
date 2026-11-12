## No CPU load

Median over rounds (up to 3), round range in small print. Lower is better, except interval p50, which is the refresh period.

### move

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> |
| interval p90 ms | 5.75 <sub>5.74–5.75</sub> | 5.76 <sub>5.75–5.76</sub> | 5.77 <sub>5.76–5.77</sub> | 5.77 <sub>5.76–5.77</sub> |
| frame p90 ms | 0.99 <sub>0.98–1.11</sub> | 1.00 <sub>0.99–1.22</sub> | 1.25 <sub>0.91–1.50</sub> | 1.05 <sub>0.90–1.27</sub> |
| frames >2x budget | 0.00 <sub>0.00–2.00</sub> | 1.00 <sub>0.00–1.00</sub> | 1.00 <sub>1.00–2.00</sub> | 1.00 <sub>0.00–1.00</sub> |
| tick ms/frame | 0.65 <sub>0.62–1.24</sub> | 0.82 <sub>0.61–1.09</sub> | 1.25 <sub>0.86–1.52</sub> | 0.96 <sub>0.88–1.22</sub> |
| peek_frame ms/frame | n/a | n/a | 0.51 <sub>0.38–0.61</sub> | 0.42 <sub>0.38–0.52</sub> |
| input->first frame ms | 16.94 <sub>16.47–45.78</sub> | 28.18 <sub>18.76–50.15</sub> | n/a | n/a |
| command p50 ms | 16.22 <sub>15.64–35.81</sub> | 23.95 <sub>18.23–42.35</sub> | 37.30 <sub>22.79–40.78</sub> | 23.99 <sub>23.48–38.14</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default |
| --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | ≈ | ≈ | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | ≈ | ≈ | ≈ | ≈ | ≈ |

### float

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> |
| interval p90 ms | 10.47 <sub>10.25–10.79</sub> | 9.18 <sub>9.01–10.18</sub> | 9.64 <sub>9.52–10.86</sub> | 6.40 <sub>6.33–6.48</sub> |
| frame p90 ms | 10.18 <sub>10.12–10.71</sub> | 8.93 <sub>8.55–9.39</sub> | 9.44 <sub>6.30–10.56</sub> | 4.38 <sub>4.16–4.89</sub> |
| frames >2x budget | 5.00 <sub>5.00–6.00</sub> | 4.00 <sub>3.00–4.00</sub> | 4.00 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 3.87 <sub>3.66–3.91</sub> | 3.61 <sub>3.40–3.68</sub> | 4.15 <sub>3.90–4.94</sub> | 5.06 <sub>4.95–5.66</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 3.32 <sub>3.28–3.51</sub> |
| input->first frame ms | 97.94 <sub>97.35–99.84</sub> | 119.26 <sub>119.23–121.69</sub> | n/a | n/a |
| command p50 ms | 79.30 <sub>77.25–81.82</sub> | 102.17 <sub>99.40–113.47</sub> | 99.28 <sub>92.17–111.12</sub> | 133.88 <sub>133.00–143.97</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default |
| --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | better (-39%) | better (-30%) | better (-34%) |
| frame p90 ms | ≈ | ≈ | better (-57%) | better (-51%) | better (-54%) |
| frames >2x budget | ≈ | ≈ | ≈ | worse (+25%) | ≈ |
| tick ms/frame | ≈ | worse (+15%) | worse (+31%) | worse (+40%) | worse (+22%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+25%) | ≈ | worse (+69%) | worse (+31%) | worse (+35%) |

### minimize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.78 <sub>5.77–5.80</sub> | 6.22 <sub>5.98–6.37</sub> | 5.77 <sub>5.77–5.78</sub> | 5.79 <sub>5.78–5.80</sub> |
| frame p90 ms | 3.86 <sub>3.05–4.45</sub> | 5.66 <sub>4.65–5.73</sub> | 4.02 <sub>3.67–4.05</sub> | 3.76 <sub>3.56–3.90</sub> |
| frames >2x budget | 6.00 <sub>5.00–6.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> | 6.00 <sub>6.00–7.00</sub> |
| tick ms/frame | 1.67 <sub>1.61–1.71</sub> | 1.56 <sub>1.40–1.75</sub> | 1.76 <sub>1.73–2.26</sub> | 2.21 <sub>1.79–2.53</sub> |
| peek_frame ms/frame | n/a | n/a | 1.20 <sub>1.18–1.20</sub> | 1.24 <sub>1.19–1.32</sub> |
| input->first frame ms | 99.00 <sub>97.50–126.44</sub> | 117.13 <sub>114.08–128.89</sub> | n/a | n/a |
| command p50 ms | 46.38 <sub>44.83–52.82</sub> | 62.12 <sub>59.82–63.57</sub> | 62.36 <sub>40.46–66.23</sub> | 68.08 <sub>67.99–68.68</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default |
| --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | better (-7%) | ≈ | better (-7%) | ≈ |
| frame p90 ms | ≈ | better (-29%) | ≈ | better (-34%) | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | worse (+20%) | worse (+20%) |
| tick ms/frame | worse (+5%) | ≈ | worse (+32%) | worse (+42%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | ≈ | ≈ | worse (+47%) | worse (+10%) | worse (+9%) |

### resize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 10.22 <sub>9.24–10.44</sub> | 9.24 <sub>9.10–10.66</sub> | 6.77 <sub>5.81–6.96</sub> | 5.98 <sub>5.90–6.34</sub> |
| frame p90 ms | 10.12 <sub>9.10–10.27</sub> | 8.80 <sub>8.14–9.83</sub> | 4.40 <sub>3.92–4.92</sub> | 4.41 <sub>4.24–4.42</sub> |
| frames >2x budget | 5.00 <sub>4.00–5.00</sub> | 4.00 <sub>4.00–4.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.14 <sub>3.93–4.31</sub> | 4.05 <sub>3.36–4.15</sub> | 5.62 <sub>5.17–5.93</sub> | 5.18 <sub>5.06–5.41</sub> |
| peek_frame ms/frame | n/a | n/a | 3.37 <sub>3.34–3.42</sub> | 3.36 <sub>3.35–3.40</sub> |
| input->first frame ms | 96.06 <sub>82.04–99.55</sub> | 117.57 <sub>114.07–117.91</sub> | n/a | n/a |
| command p50 ms | 75.03 <sub>73.29–76.37</sub> | 103.26 <sub>97.10–107.89</sub> | 134.21 <sub>133.39–135.46</sub> | 141.68 <sub>125.53–141.77</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default |
| --- | --- | --- | --- | --- | --- |
| interval p90 ms | better (-34%) | better (-27%) | better (-41%) | better (-35%) | ≈ |
| frame p90 ms | better (-57%) | better (-50%) | better (-56%) | better (-50%) | ≈ |
| frames >2x budget | ≈ | worse (+25%) | ≈ | worse (+25%) | ≈ |
| tick ms/frame | worse (+36%) | worse (+39%) | worse (+25%) | worse (+28%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+79%) | worse (+30%) | worse (+89%) | worse (+37%) | ≈ |

### switch

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.75 <sub>5.75–5.76</sub> | 5.76 <sub>5.76–5.79</sub> | 5.77 <sub>5.77–5.78</sub> | 5.77 <sub>5.76–5.77</sub> |
| frame p90 ms | 3.52 <sub>3.34–3.54</sub> | 1.90 <sub>1.84–1.92</sub> | 1.75 <sub>1.69–2.03</sub> | 1.94 <sub>1.55–2.27</sub> |
| frames >2x budget | 3.00 <sub>2.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> | 3.00 <sub>3.00–4.00</sub> | 4.00 <sub>3.00–4.00</sub> |
| tick ms/frame | 2.50 <sub>2.22–2.51</sub> | 1.95 <sub>1.93–2.05</sub> | 1.98 <sub>1.68–2.09</sub> | 1.83 <sub>1.65–2.17</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> |
| input->first frame ms | n/a | n/a | n/a | n/a |
| command p50 ms | 50.11 <sub>28.64–51.16</sub> | 66.86 <sub>55.89–77.90</sub> | 68.97 <sub>51.18–80.76</sub> | 70.26 <sub>58.22–71.60</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default |
| --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-50%) | ≈ | better (-45%) | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | better (-21%) | ≈ | better (-27%) | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+38%) | ≈ | worse (+40%) | ≈ | ≈ |
