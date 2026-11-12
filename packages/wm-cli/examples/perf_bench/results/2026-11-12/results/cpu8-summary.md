## With CPU load (-CpuLoad 8)

Median over rounds (up to 3), round range in small print. Lower is better, except interval p50, which is the refresh period.

### move

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.71</sub> | 5.71 <sub>5.71–5.71</sub> |
| interval p90 ms | 5.76 <sub>5.75–5.77</sub> | 5.75 <sub>5.74–5.76</sub> | 5.75 <sub>5.75–5.77</sub> | 5.76 <sub>5.76–5.77</sub> |
| frame p90 ms | 1.16 <sub>1.10–1.22</sub> | 1.18 <sub>1.08–1.19</sub> | 1.01 <sub>0.99–1.53</sub> | 1.37 <sub>0.63–1.53</sub> |
| frames >2x budget | 1.00 <sub>0.00–1.00</sub> | 1.00 <sub>0.00–1.00</sub> | 1.00 <sub>1.00–1.00</sub> | 1.00 <sub>1.00–1.00</sub> |
| tick ms/frame | 0.79 <sub>0.74–1.38</sub> | 0.64 <sub>0.61–1.23</sub> | 1.01 <sub>0.97–1.38</sub> | 1.16 <sub>0.92–1.18</sub> |
| peek_frame ms/frame | n/a | n/a | 0.47 <sub>0.41–0.61</sub> | 0.55 <sub>0.46–0.58</sub> |
| input->first frame ms | 17.91 <sub>17.24–49.80</sub> | 20.44 <sub>19.69–50.14</sub> | n/a | n/a |
| command p50 ms | 17.08 <sub>16.42–39.48</sub> | 19.73 <sub>18.99–38.58</sub> | 24.84 <sub>24.47–41.74</sub> | 28.59 <sub>24.62–28.82</sub> |

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
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.71 <sub>5.71–5.71</sub> |
| interval p90 ms | 11.07 <sub>11.06–11.53</sub> | 10.17 <sub>10.02–10.58</sub> | 10.78 <sub>9.62–11.18</sub> | 7.03 <sub>6.92–8.27</sub> |
| frame p90 ms | 8.87 <sub>7.96–11.27</sub> | 9.92 <sub>9.57–10.49</sub> | 10.02 <sub>8.77–10.92</sub> | 5.52 <sub>4.84–5.99</sub> |
| frames >2x budget | 6.00 <sub>5.00–6.00</sub> | 4.00 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.25 <sub>3.99–4.34</sub> | 3.82 <sub>3.43–4.34</sub> | 4.47 <sub>4.26–5.08</sub> | 5.83 <sub>5.69–5.90</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 3.68 <sub>3.39–3.73</sub> |
| input->first frame ms | 109.78 <sub>76.03–116.91</sub> | 129.00 <sub>125.91–135.11</sub> | n/a | n/a |
| command p50 ms | 87.41 <sub>85.87–88.50</sub> | 111.66 <sub>111.36–117.42</sub> | 108.57 <sub>96.66–122.85</sub> | 152.81 <sub>145.73–152.90</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default |
| --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | better (-36%) | better (-31%) | better (-35%) |
| frame p90 ms | ≈ | ≈ | better (-38%) | better (-44%) | better (-45%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | ≈ | ≈ | worse (+37%) | worse (+53%) | worse (+30%) |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+24%) | ≈ | worse (+75%) | worse (+37%) | worse (+41%) |

### minimize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.72–5.72</sub> |
| interval p90 ms | 5.83 <sub>5.82–6.52</sub> | 6.10 <sub>5.82–6.50</sub> | 5.79 <sub>5.78–5.81</sub> | 5.79 <sub>5.78–5.84</sub> |
| frame p90 ms | 5.13 <sub>3.77–6.46</sub> | 5.09 <sub>3.22–5.56</sub> | 4.55 <sub>4.48–4.81</sub> | 4.36 <sub>4.04–4.81</sub> |
| frames >2x budget | 8.00 <sub>7.00–8.00</sub> | 6.00 <sub>5.00–7.00</sub> | 7.00 <sub>7.00–7.00</sub> | 7.00 <sub>6.00–7.00</sub> |
| tick ms/frame | 2.04 <sub>1.88–2.06</sub> | 1.72 <sub>1.65–1.84</sub> | 2.51 <sub>2.09–2.58</sub> | 2.14 <sub>2.09–2.15</sub> |
| peek_frame ms/frame | n/a | n/a | 1.31 <sub>1.24–1.34</sub> | 0.76 <sub>0.00–0.80</sub> |
| input->first frame ms | 127.90 <sub>98.45–128.63</sub> | 126.39 <sub>112.38–138.99</sub> | n/a | n/a |
| command p50 ms | 53.55 <sub>27.24–54.87</sub> | 69.12 <sub>68.87–73.04</sub> | 75.62 <sub>68.57–82.54</sub> | 77.28 <sub>61.09–78.72</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default |
| --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | better (-5%) | ≈ | ≈ | ≈ |
| frame p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | worse (+23%) | worse (+46%) | ≈ | worse (+24%) | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+41%) | ≈ | worse (+44%) | ≈ | ≈ |

### resize

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.72–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> |
| interval p90 ms | 11.84 <sub>11.16–12.57</sub> | 10.24 <sub>8.92–10.56</sub> | 6.64 <sub>6.53–7.08</sub> | 6.13 <sub>5.85–7.18</sub> |
| frame p90 ms | 11.75 <sub>10.91–12.50</sub> | 10.00 <sub>8.81–10.14</sub> | 5.38 <sub>5.36–5.43</sub> | 5.15 <sub>4.98–5.22</sub> |
| frames >2x budget | 6.00 <sub>5.00–6.00</sub> | 5.00 <sub>4.00–5.00</sub> | 5.00 <sub>4.00–5.00</sub> | 5.00 <sub>5.00–5.00</sub> |
| tick ms/frame | 4.77 <sub>4.68–4.87</sub> | 4.27 <sub>3.95–4.55</sub> | 6.10 <sub>6.07–6.57</sub> | 6.00 <sub>4.54–6.02</sub> |
| peek_frame ms/frame | n/a | n/a | 3.56 <sub>3.40–3.70</sub> | 3.57 <sub>1.74–3.61</sub> |
| input->first frame ms | 104.99 <sub>95.85–105.06</sub> | 129.74 <sub>117.83–133.20</sub> | n/a | n/a |
| command p50 ms | 82.66 <sub>80.87–84.12</sub> | 108.71 <sub>107.93–110.68</sub> | 151.08 <sub>147.60–159.33</sub> | 146.15 <sub>133.31–149.08</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default |
| --- | --- | --- | --- | --- | --- |
| interval p90 ms | better (-44%) | better (-35%) | better (-48%) | better (-40%) | ≈ |
| frame p90 ms | better (-54%) | better (-46%) | better (-56%) | better (-48%) | ≈ |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | worse (+28%) | worse (+43%) | ≈ | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+83%) | worse (+39%) | worse (+77%) | worse (+34%) | ≈ |

### switch

| metric | main | nopeeks | default | morepeeks |
| --- | ---: | ---: | ---: | ---: |
| interval p50 ms | 5.71 <sub>5.71–5.72</sub> | 5.71 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> | 5.72 <sub>5.71–5.72</sub> |
| interval p90 ms | 5.78 <sub>5.77–5.79</sub> | 5.77 <sub>5.76–5.77</sub> | 5.78 <sub>5.77–5.78</sub> | 5.77 <sub>5.77–5.78</sub> |
| frame p90 ms | 4.08 <sub>3.69–4.15</sub> | 2.16 <sub>2.01–2.18</sub> | 2.22 <sub>2.13–2.35</sub> | 2.03 <sub>1.95–2.12</sub> |
| frames >2x budget | 2.00 <sub>2.00–3.00</sub> | 3.00 <sub>3.00–3.00</sub> | 4.00 <sub>3.00–4.00</sub> | 3.00 <sub>3.00–4.00</sub> |
| tick ms/frame | 2.83 <sub>2.49–2.85</sub> | 1.94 <sub>1.89–2.22</sub> | 2.20 <sub>2.03–2.44</sub> | 2.06 <sub>2.01–2.20</sub> |
| peek_frame ms/frame | n/a | n/a | 0.00 <sub>0.00–0.00</sub> | 0.00 <sub>0.00–0.00</sub> |
| input->first frame ms | n/a | n/a | n/a | n/a |
| command p50 ms | 40.88 <sub>31.17–47.70</sub> | 80.86 <sub>73.45–85.57</sub> | 74.70 <sub>61.29–84.62</sub> | 86.06 <sub>55.72–86.08</sub> |

| vs (ranges must not overlap) | default vs main | default vs nopeeks | morepeeks vs main | morepeeks vs nopeeks | morepeeks vs default |
| --- | --- | --- | --- | --- | --- |
| interval p90 ms | ≈ | ≈ | ≈ | ≈ | ≈ |
| frame p90 ms | better (-46%) | ≈ | better (-50%) | ≈ | better (-9%) |
| frames >2x budget | ≈ | ≈ | ≈ | ≈ | ≈ |
| tick ms/frame | better (-22%) | ≈ | better (-27%) | ≈ | ≈ |
| input->first frame ms | n/a | n/a | n/a | n/a | n/a |
| command p50 ms | worse (+83%) | ≈ | worse (+111%) | ≈ | ≈ |
