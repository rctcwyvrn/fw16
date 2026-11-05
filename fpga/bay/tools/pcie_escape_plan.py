"""PCIe escape placement (approved 2026-11-05). Stage 1: J1 via columns + L1 stubs, U14 GT_TX dogbones, TX0/TX2 drops.
Stage 2 (all TX lanes on L3): TX1/TX3 drops, TX1 J1 vias, AC-cap hop vias + B.Cu stubs (caps C77-C84 on B.Cu, GT pad
to the right). usage: pcie_escape_plan.py OUT.json [2]. Writes the chosen stage's ops and checks clearances against existing copper (shapely)."""
import json, sys, board_copper as g
from shapely.geometry import Point, LineString, box
VIA=0.45; DR=0.2
vias=[]; segs=[]
def via(x,y,n): vias.append(dict(x=round(x,3),y=round(y,3),size=VIA,drill=DR,net=n))
def seg(pts,n,w=0.1,layer='F.Cu'):
    for (x1,y1),(x2,y2) in zip(pts,pts[1:]): segs.append(dict(x1=x1,y1=y1,x2=x2,y2=y2,w=w,layer=layer,net=n))
# J1 via columns
A=[(-3.13,'PEX_RX1_N',1.6),(-2.53,'PEX_RX1_P',1.85),(-1.93,'GND',1.85),(-1.33,'PEX_RX0_N',1.85),(-0.73,'PEX_RX0_P',1.85),
   (-0.13,'GND',1.85),(0.47,'PEX_TX0_N',1.85),(1.07,'PEX_TX0_P',1.85),(1.67,'GND',1.85)]
for y,n,x in A: via(x,y,n); seg([(x,y),(2.945,y)],n,0.2 if n=='GND' else 0.1)
for y,n in [(-2.53,'GND'),(-0.73,'GND'),(1.07,'GND'),(1.67,'DP_A_AUX#'),(2.27,'DP_A_AUX'),(2.87,'GPIO3_EC')]:
    via(4.55,y,n); seg([(3.81,y),(4.55,y)],n,0.2 if n=='GND' else 0.1)
for y,n in [(-3.13,'PEX_RX3_N'),(-2.53,'PEX_RX3_P'),(-1.93,'GND'),(-1.33,'PEX_RX2_N'),(-0.73,'PEX_RX2_P'),(-0.13,'GND'),
            (0.47,'PEX_TX2_P'),(1.07,'PEX_TX2_N'),(1.67,'GND'),(2.27,'PEX_TX3_N'),(2.87,'PEX_TX3_P')]:
    via(5.5,y,n); seg([(5.5,y),(6.25,y)],n,0.2 if n=='GND' else 0.1)
# U14 GT_TX dogbones (rows 4/5), site = ball + (0.389, 0.389) like the existing fanout
for i,x in enumerate([1.5,3.5,5.5,7.5]):
    for yb,pol in [(-31.55,'N'),(-32.55,'P')]:
        n=f'/FPGA/GT_TX{i}_{pol}'; vx,vy=round(x+0.389,3),round(yb+0.389,3); via(vx,vy,n); seg([(x,yb),(vx,vy)],n)
# TX0 / TX2 (host -> FPGA RX) drop to L3 just outside the ball field; polarity order fixed here
via(0.5,-27.7,'PEX_TX0_N'); seg([(0.5,-28.55),(0.5,-27.7)],'PEX_TX0_N')
via(0.0,-27.0,'PEX_TX0_P'); seg([(0.5,-29.55),(0.0,-29.05),(0.0,-27.0)],'PEX_TX0_P')
via(4.5,-27.7,'PEX_TX2_N'); seg([(4.5,-28.55),(4.5,-27.7)],'PEX_TX2_N')
via(5.0,-27.0,'PEX_TX2_P'); seg([(4.5,-29.55),(5.0,-29.05),(5.0,-27.0)],'PEX_TX2_P')
STAGE2 = len(sys.argv) > 2 and sys.argv[2] == '2'
if STAGE2:
    vias.clear(); segs.clear()
    for x, n in [(2.5, 'PEX_TX1'), (6.5, 'PEX_TX3')]:
        via(x, -27.7, n + '_N'); seg([(x, -28.55), (x, -27.7)], n + '_N')
        via(x + 0.5, -27.0, n + '_P'); seg([(x, -29.55), (x + 0.5, -29.05), (x + 0.5, -27.0)], n + '_P')
    for y, n in [(2.27, 'PEX_TX1_N'), (2.87, 'PEX_TX1_P')]:
        via(1.85, y, n); seg([(1.85, y), (2.945, y)], n)
    # cap hops: (lane, cap centre x, N row y, GT via x, PEX via x); pads at xc +/- 0.48, P row = N row + 1
    for i, xc, yn, xg, xp in [(0, -2.9, -23.0, -1.8, -4.0), (1, -0.3, -19.5, 1.4, -2.0),
                              (2, 5.1, -23.0, 6.2, 4.0), (3, 7.8, -19.5, 9.6, 6.0)]:
        for pol, y in [('N', yn), ('P', yn + 1.0)]:
            gt, pex = f'/FPGA/GT_TX{i}_{pol}', f'PEX_RX{i}_{pol}'
            via(xg, y, gt); seg([(xg, y), (xc + 0.48, y)], gt, 0.1, 'B.Cu')
            via(xp, y, pex); seg([(xp, y), (xc - 0.48, y)], pex, 0.1, 'B.Cu')
json.dump(dict(vias=vias,segments=segs),open(sys.argv[1] if len(sys.argv)>1 else 'ops.json','w'),indent=0)

# clearance check: new items vs existing copper and vs each other (different nets)
CL=0.1; HOLE=0.25
items=[]  # (geom, net, layers, kind)
for p in g.pads():
    w,h=(p['sx'],p['sy']) if p['rot']%180==0 else (p['sy'],p['sx'])
    geo=Point(p['x'],p['y']).buffer(w/2) if p['shape']=='circle' else box(p['x']-w/2,p['y']-h/2,p['x']+w/2,p['y']+h/2)
    lay={'F.Cu','In1.Cu','In2.Cu','In3.Cu','In4.Cu','In5.Cu','In6.Cu','B.Cu'} if '*.Cu' in p['layers'] else {l for l in p['layers'] if l.endswith('.Cu')}
    items.append((geo,p['net'],lay,'npth' if p['kind']=='np_thru_hole' else 'pad'))
ALL={'F.Cu','In1.Cu','In2.Cu','In3.Cu','In4.Cu','In5.Cu','In6.Cu','B.Cu'}
for v in g.vias(): items.append((Point(v['x'],v['y']).buffer(v['size']/2),v['net'],ALL,'via'))
for s in g.segments(): items.append((LineString([(s['x1'],s['y1']),(s['x2'],s['y2'])]).buffer(s['w']/2),s['net'],{s['layer']},'seg'))
new=[(Point(v['x'],v['y']).buffer(VIA/2),v['net'],ALL,'newvia') for v in vias]+\
    [(LineString([(s['x1'],s['y1']),(s['x2'],s['y2'])]).buffer(s['w']/2),s['net'],{s['layer']},'newseg') for s in segs]
bad=0
for i,(ga,na,la,ka) in enumerate(new):
    for gb,nb,lb,kb in items+new[i+1:]:
        if na==nb or not (la&lb): continue
        need=HOLE if 'npth' in (ka,kb) else CL
        d=ga.distance(gb)
        if d<need-1e-6:
            bad+=1; print(f'CLEAR {ka} {na} vs {kb} {nb}: {d:.3f} < {need}', [round(c,2) for c in ga.centroid.coords[0]])
print(len(vias),'vias',len(segs),'segments; violations',bad)
