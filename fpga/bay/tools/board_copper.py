"""Extract copper geometry (pads, vias, segments) from the card board for escape planning."""
import re, math
import os
B = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "card", "fw16-fpga-module.kicad_pcb")
b = open(B).read()
def fps():
    starts=[m.start() for m in re.finditer(r'\n\t\(footprint "',b)]+[len(b)]
    for a,c in zip(starts,starts[1:]):
        blk=b[a:c]
        if '(footprint' not in blk[:20]: continue
        ref=re.search(r'\(property "Reference" "([^"]+)"',blk).group(1)
        layer=re.search(r'\(footprint "[^"]+"\s*\(layer "([^"]+)"\)',blk).group(1)
        x,y,r=re.search(r'\(at ([-\d.]+) ([-\d.]+)(?: ([-\d.]+))?\)',blk).groups()
        yield ref,layer,float(x),float(y),float(r or 0),blk
def pads():
    out=[]
    for ref,fl,x,y,r,blk in fps():
        t=math.radians(r)
        for p in blk.split('\n\t\t(pad ')[1:]:
            nm=p.split()[0].strip('"'); kind=p.split()[1]; shape=p.split()[2]
            m=re.search(r'\(at ([-\d.]+) ([-\d.]+)(?: ([-\d.]+))?\)',p); lx,ly=float(m.group(1)),float(m.group(2)); pr=float(m.group(3) or 0)
            sx,sy=map(float,re.search(r'\(size ([\d.]+) ([\d.]+)',p).groups())
            ly_=re.search(r'\(layers ([^)]*)\)',p).group(1).replace('"','').split()
            drill=re.search(r'\(drill (?:oval )?([\d.]+)',p); drill=float(drill.group(1)) if drill else 0
            n=re.search(r'\(net (?:\d+ )?"([^"]+)"\)',p); n=n.group(1) if n else ''
            bx=x+lx*math.cos(t)+ly*math.sin(t); by=y-lx*math.sin(t)+ly*math.cos(t)
            out.append(dict(ref=ref,pad=nm,kind=kind,shape=shape,x=bx,y=by,sx=sx,sy=sy,rot=pr,layers=ly_,drill=drill,net=n))
    return out
def vias():
    out=[]
    for m in re.finditer(r'\(via\s*\(at ([-\d.]+) ([-\d.]+)\)\s*\(size ([\d.]+)\)\s*\(drill ([\d.]+)\).*?\(net (?:\d+ )?"?([^")]*)"?\)',b,re.S):
        out.append(dict(x=float(m.group(1)),y=float(m.group(2)),size=float(m.group(3)),drill=float(m.group(4)),net=m.group(5)))
    return out
def segments():
    out=[]
    for m in re.finditer(r'\(segment\s*\(start ([-\d.]+) ([-\d.]+)\)\s*\(end ([-\d.]+) ([-\d.]+)\)\s*\(width ([\d.]+)\)\s*\(layer "([^"]+)"\).*?\(net (?:\d+ )?"?([^")]*)"?\)',b,re.S):
        out.append(dict(x1=float(m.group(1)),y1=float(m.group(2)),x2=float(m.group(3)),y2=float(m.group(4)),w=float(m.group(5)),layer=m.group(6),net=m.group(7)))
    return out
