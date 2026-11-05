"""Render a region of the card's copper (pads, vias, tracks, planned items) to SVG/PNG for review."""
import sys, json, board_copper as g
X0,Y0,X1,Y1 = map(float, sys.argv[1:5]); out=sys.argv[5]; plan=json.load(open(sys.argv[6])) if len(sys.argv)>6 else {}
S=60  # px/mm
col={'F.Cu':'#c83232','In2.Cu':'#3264c8','B.Cu':'#32a032'}
def X(x): return (x-X0)*S
def Y(y): return (y-Y0)*S
o=[f'<svg xmlns="http://www.w3.org/2000/svg" width="{(X1-X0)*S}" height="{(Y1-Y0)*S}" style="background:#111">']
def lab(x,y,t,c='#ddd',sz=7): o.append(f'<text x="{X(x)}" y="{Y(y)}" fill="{c}" font-size="{sz}" text-anchor="middle" font-family="monospace">{t}</text>')
for p in g.pads():
    if not (X0-1<p['x']<X1+1 and Y0-1<p['y']<Y1+1): continue
    f='F.Cu' in p['layers'] or '*.Cu' in p['layers']
    c='#a05050' if f else '#50a050'
    if p['kind']=='np_thru_hole': c='#666'
    w,h=(p['sx'],p['sy']) if p['rot']%180==0 else (p['sy'],p['sx'])
    if p['shape'] in('circle',) : o.append(f'<circle cx="{X(p["x"])}" cy="{Y(p["y"])}" r="{w/2*S}" fill="{c}" opacity="0.8"/>')
    else: o.append(f'<rect x="{X(p["x"]-w/2)}" y="{Y(p["y"]-h/2)}" width="{w*S}" height="{h*S}" fill="{c}" opacity="0.8"/>')
    n=p['net'].replace('/FPGA/','').replace('unconnected-','').split('-Pad')[0][-9:]
    if n: lab(p['x'],p['y']+0.05,n)
for v in g.vias():
    if X0<v['x']<X1 and Y0<v['y']<Y1:
        o.append(f'<circle cx="{X(v["x"])}" cy="{Y(v["y"])}" r="{v["size"]/2*S}" fill="#c8a000"/>'); lab(v['x'],v['y']+0.05,v['net'][-6:],'#000',6)
for s in g.segments()+plan.get('segments',[]):
    if not(X0-2<s['x1']<X1+2 and Y0-2<s['y1']<Y1+2): continue
    o.append(f'<line x1="{X(s["x1"])}" y1="{Y(s["y1"])}" x2="{X(s["x2"])}" y2="{Y(s["y2"])}" stroke="{col.get(s["layer"],"#aaa")}" stroke-width="{s["w"]*S}" stroke-linecap="round" opacity="0.85"/>')
for v in plan.get('vias',[]):
    o.append(f'<circle cx="{X(v["x"])}" cy="{Y(v["y"])}" r="{v["size"]/2*S}" fill="#ffe060" stroke="#fff" stroke-width="1"/>')
for gx in range(int(X0),int(X1)+1): lab(gx,Y0+0.3,str(gx),'#888',8)
for gy in range(int(Y0)+1,int(Y1)+1): o.append(f'<text x="2" y="{Y(gy)}" fill="#888" font-size="8">{gy}</text>')
o.append('</svg>'); open(out+'.svg','w').write('\n'.join(o))
