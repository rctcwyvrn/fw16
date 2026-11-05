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
    for m in re.finditer(r'\(segment\s*\(start ([-\d.]+) ([-\d.]+)\)\s*\(end ([-\d.]+) ([-\d.]+)\)\s*\(width ([\d.]+)\)\s*\(layer "([^"]+)"\).*?\(net (?:\d+ )?"?([^")]*)"?\)\s*(?:\(uuid "([^"]+)"\))?',b,re.S):
        out.append(dict(x1=float(m.group(1)),y1=float(m.group(2)),x2=float(m.group(3)),y2=float(m.group(4)),w=float(m.group(5)),layer=m.group(6),net=m.group(7),uuid=m.group(8)))
    return out

def outline(arc_steps=12):
    """Every closed Edge.Cuts loop as a list of (x, y): lines and three-point arcs chained end to end (0.01 mm)."""
    import math
    segs = []
    blocks = [blk for blk in re.split(r'\n\t(?=\(gr_)', b) if blk.startswith('(gr_') and '(layer "Edge.Cuts")' in blk.split('\n\t(')[0] + blk[:400]]
    for blk in blocks:
        m = re.match(r'\(gr_(line|arc)\s*\(start ([-\d.]+) ([-\d.]+)\)\s*(?:\(mid ([-\d.]+) ([-\d.]+)\)\s*)?\(end ([-\d.]+) ([-\d.]+)\)', blk)
        if not m or '(layer "Edge.Cuts")' not in blk[:400]: continue
        s = (float(m.group(2)), float(m.group(3))); e = (float(m.group(6)), float(m.group(7)))
        if m.group(1) == 'line':
            segs.append([s, e]); continue
        mid = (float(m.group(4)), float(m.group(5)))
        (ax, ay), (bx, by), (cx, cy) = s, mid, e
        d = 2 * (ax * (by - cy) + bx * (cy - ay) + cx * (ay - by))
        ux = ((ax*ax+ay*ay)*(by-cy)+(bx*bx+by*by)*(cy-ay)+(cx*cx+cy*cy)*(ay-by))/d
        uy = ((ax*ax+ay*ay)*(cx-bx)+(bx*bx+by*by)*(ax-cx)+(cx*cx+cy*cy)*(bx-ax))/d
        a0, am, a1 = (math.atan2(p[1]-uy, p[0]-ux) for p in (s, mid, e))
        sw = (a1 - a0) % (2*math.pi)
        if (am - a0) % (2*math.pi) > sw: sw -= 2*math.pi
        r = math.hypot(ax-ux, ay-uy)
        pts = [(ux + r*math.cos(a0 + sw*i/arc_steps), uy + r*math.sin(a0 + sw*i/arc_steps)) for i in range(arc_steps+1)]
        segs.append([s] + pts[1:-1] + [e])   # exact endpoints, so chaining never meets rounding noise
    key = lambda p: (round(p[0], 3), round(p[1], 3))
    loops = []
    while segs:
        loop = segs.pop(0)
        while key(loop[0]) != key(loop[-1]):
            for i, sg in enumerate(segs):
                if key(sg[0]) == key(loop[-1]): loop += sg[1:]; break
                if key(sg[-1]) == key(loop[-1]): loop += sg[::-1][1:]; break
            else:
                raise SystemExit('Edge.Cuts has an open chain')
            segs.pop(i)
        loops.append(loop)
    return loops   # outer board loop first? no: callers pick by area
