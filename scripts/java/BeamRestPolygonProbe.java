// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.awt.*;
import java.awt.geom.*;
import java.lang.reflect.*;
import java.util.*;
import java.util.List;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Declared cached geometry on real timed Chords; private original lookup is the acceptance oracle. */
public class BeamRestPolygonProbe extends BeamRestClearProbe {
    static void field(Object object,String name,Object value)throws Exception {
        Class<?> type=object.getClass();while(type!=null){try{Field f=type.getDeclaredField(name);f.setAccessible(true);f.set(object,value);return;}catch(NoSuchFieldException e){type=type.getSuperclass();}}throw new NoSuchFieldException(name);
    }
    static List<Integer> coords(Point p){return List.of(p.x,p.y);}
    static Point point(List<Integer> p){return new Point(p.get(0),p.get(1));}
    static Polygon polygon(Object group,Object left,Object right)throws Exception {
        Point lh=(Point)call(left,"getHeadLocation"),lt=(Point)call(left,"getTailLocation"),rh=(Point)call(right,"getHeadLocation"),rt=(Point)call(right,"getTailLocation");int ld=Integer.signum(lh.y-lt.y),rd=Integer.signum(rh.y-rt.y);Polygon p=new Polygon();
        if(ld==rd){Object ls=call(left,"getStaff");if(ls!=null&&ls==call(right,"getStaff")){Object line=call(ls,"getMidLine");int ml=(Integer)line.getClass().getMethod("yAt",int.class).invoke(line,lh.x),mr=(Integer)line.getClass().getMethod("yAt",int.class).invoke(line,rh.x);if(ld*(ml-lh.y)>0)lh.y=ml;if(rd*(mr-rh.y)>0)rh.y=mr;}p.addPoint(rh.x,rh.y);p.addPoint(rt.x,rt.y);p.addPoint(lt.x,lt.y);p.addPoint(lh.x,lh.y);}
        else{Point uh=ld<0?lh:rh,ut=ld<0?lt:rt,dh=ld>0?lh:rh,dt=ld>0?lt:rt;Object staff=call(ld<0?left:right,"getStaff"),line=call(staff,"getMidLine");int mid=(Integer)line.getClass().getMethod("yAt",int.class).invoke(line,(lh.x+rh.x)/2);if(uh.y>mid)uh.y=mid;Method main=c("sig.inter.BeamGroupInter").getDeclaredMethod("getMainMedian");main.setAccessible(true);Object m=main.invoke(group);Point um=(Point)callStatic(c("math.PointUtil"),"rounded",callStatic(c("math.LineUtil"),"intersection",m,new Line2D.Double(uh,ut))),dm=(Point)callStatic(c("math.PointUtil"),"rounded",callStatic(c("math.LineUtil"),"intersection",m,new Line2D.Double(dh,dt)));p.addPoint(uh.x,uh.y);p.addPoint(um.x,um.y);p.addPoint(dm.x,dm.y);p.addPoint(dm.x,dm.y-(um.y-uh.y));}return p;
    }
    static Object callStatic(Class<?> cls,String name,Object... args)throws Exception {
        for(Method m:cls.getMethods())if(m.getName().equals(name)&&Modifier.isStatic(m.getModifiers())&&m.getParameterCount()==args.length){try{return m.invoke(null,args);}catch(IllegalArgumentException e){}}
        throw new NoSuchMethodException(name);
    }
    static List<Object> points(Polygon p){List<Object> rows=new ArrayList<>();for(int i=0;i<p.npoints;i++)rows.add(List.of(p.xpoints[i],p.ypoints[i]));return rows;}
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short","excess")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            List<?> stacks=(List<?>)call(system,"getStacks");for(Object s:stacks)call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",s);
            Object stack=stacks.get(0),measure=((List<?>)call(stack,"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt((Object ch)->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}).thenComparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).y;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();for(int i=0;i<chords.size();i++){Object ch=chords.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));int j=0;for(Object n:(Collection<?>)call(ch,"getMembers"))labels.put(n,"head"+i+"_"+j++);}
            List<Object> voices=new ArrayList<>((Collection<?>)call(measure,"getVoices"));for(int i=0;i<voices.size();i++)labels.put(voices.get(i),"voice"+i);
            Object old=voices.get(0),seed=call(old,"getFirstChord"),last=call(old,"getLastChord"),offset=mode.equals("short")?make("math.Rational",1,8):make("math.Rational",5,4);call(last,"setTimeOffset",offset);call(old,"checkDuration");call(seed,"setManual",true);
            Object group=call(seed,"getBeamGroup"),source=((List<?>)call(group,"getBeams")).get(0);labels.put(group,"source_group");labels.put(source,"source_beam");
            List<Object> oldRelations=new ArrayList<>();for(Object r:(Collection<?>)call(sig,"edgeSet")){labels.put(r,"relation"+oldRelations.size());oldRelations.add(r);}

            List<?> groupChords=(List<?>)call(group,"getChords");Object left=groupChords.get(0),right=groupChords.get(1);
            Map<String,Object> automatic=obj("left_head",coords((Point)call(left,"getHeadLocation")),"left_tail",coords((Point)call(left,"getTailLocation")),"right_head",coords((Point)call(right,"getHeadLocation")),"right_tail",coords((Point)call(right,"getTailLocation")));
            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),declaredRests=new ArrayList<>();
            for(int x:new int[]{292,293,300,304,305,420,421,422,430,470})for(int y:new int[]{160,180,223,243}){
                Object rest=make("sig.inter.RestInter",glyph(x,y),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,call(system,"getFirstStaff"),0.),chord=make("sig.inter.RestChordInter",.8);call(chord,"setStaff",call(system,"getFirstStaff"));call(sig,"addVertex",rest);call(sig,"addVertex",chord);call(chord,"addMember",rest);call(measure,"addInter",chord);int i=rests.size();labels.put(rest,"rest"+i);labels.put(chord,"rest_chord"+i);rests.add(chord);chords.add(chord);slots.put(chord,null);declaredRests.add(obj("bounds",List.of(x,y,8,20),"manual",false,"source_measure",true,"beam_link",false));
            }
            List<Object> declaredBeams=new ArrayList<>();for(String type:List.of("BeamHookInter","SmallBeamInter")){
                Object b=make("sig.inter."+type,make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(270,185.5,520,185.5),4.);call(sig,"addVertex",b);call(group,"addMember",b);labels.put(b,type);declaredBeams.add(obj("label",type,"kind",type,"median",List.of(270.,185.5,520.,185.5),"height",4.));
            }
            List<Object> before=new ArrayList<>(List.of(frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));
            int[][][] controls={{{300,220},{300,180},{430,225},{430,180}},{{300,270},{300,300},{430,268},{430,300}},{{300,270},{304,300},{430,225},{426,180}},{{300,225},{304,180},{430,270},{426,300}},{{300,225},{300,225},{430,225},{430,180}},{{300,225},{300,225},{430,225},{430,225}},{{300,220},{430,180},{430,225},{300,180}}};
            Method lookup=c("sig.inter.BeamGroupInter").getDeclaredMethod("lookupRests",c("sig.inter.AbstractChordInter"),c("sig.inter.AbstractChordInter"),Set.class,Set.class);lookup.setAccessible(true);
            List<Object> geometry=new ArrayList<>();
            for(int[][] input:controls){field(left,"headLocation",new Point(input[0][0],input[0][1]));field(left,"tailLocation",new Point(input[1][0],input[1][1]));field(left,"bounds",new Rectangle(292,200,13,44));field(right,"headLocation",new Point(input[2][0],input[2][1]));field(right,"tailLocation",new Point(input[3][0],input[3][1]));field(right,"bounds",new Rectangle(421,205,13,39));Polygon p=polygon(group,left,right);List<Object> rows=new ArrayList<>();
                for(Object rest:rests){Rectangle box=(Rectangle)call(rest,"getBounds");Point center=(Point)call(((List<?>)call(rest,"getMembers")).get(0),"getCenter");List<Object> edges=new ArrayList<>((Collection<?>)call(sig,"edgeSet"));lookup.invoke(group,left,right,new LinkedHashSet<>(List.of(rest)),new LinkedHashSet<>());List<Object> fresh=new ArrayList<>((Collection<?>)call(sig,"edgeSet"));fresh.removeAll(edges);rows.add(obj("rest",labels.get(rest),"intersects",p.intersects(box.x,box.y,box.width,box.height),"x_overlap",(Integer)callStatic(c("math.GeoUtil"),"xOverlap",box,call(left,"getBounds"))>0||(Integer)callStatic(c("math.GeoUtil"),"xOverlap",box,call(right,"getBounds"))>0,"accepted",!fresh.isEmpty()));for(Object edge:fresh)call(sig,"removeEdge",edge);}
                geometry.add(obj("input",input,"left_bounds",List.of(292,200,13,44),"right_bounds",List.of(421,205,13,39),"polygon",points(p),"main_median",List.of(270.,185.5,520.,185.5),"queries",rows));
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"left",labels.get(left),"right",labels.get(right),"automatic_cached_geometry",automatic,"declared_rests",declaredRests,"declared_beams",declaredBeams,"geometry",geometry,"before",before.get(0),"after",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));
        }
        List<Object> rectangleControls=new ArrayList<>();int[][][] polygons={{{0,0},{10,0},{10,10},{0,10}},{{0,0},{10,10},{0,10},{10,0}},{{0,0},{10,0},{10,10},{0,10},{0,0},{10,0},{10,10},{0,10}},{{0,5},{10,5},{20,5}},{{0,0},{0,10},{1,10},{1,0}}};
        for(int[][] ps:polygons){Polygon p=new Polygon();for(int[] pt:ps)p.addPoint(pt[0],pt[1]);for(int x:new int[]{-2,0,1,5,10})for(int y:new int[]{-2,0,1,5,10})for(int w:new int[]{0,1,2,10})rectangleControls.add(obj("polygon",ps,"bounds",List.of(x,y,w,2),"intersects",p.intersects(x,y,w,2)));}
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original private lookup acceptance on actual timed Chords with declared cached geometry; automatic source geometry retained separately; actual Staff identity/middle line/main Hook before equal-width Small; Polygon even-odd rectangles/contact; no archive","cases",cases,"rectangle_controls",rectangleControls));
    }
}
