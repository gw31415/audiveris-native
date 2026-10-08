// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.awt.Point;
import java.awt.geom.Line2D;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Real group nearest queries, retaining the original timed owners. */
public class BeamRestNearestProbe extends BeamRestClearProbe {
    static Object beam(String kind,double[] m)throws Exception {
        return make("sig.inter."+kind,make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),
            new Line2D.Double(m[0],m[1],m[2],m[3]),4.);
    }
    static Object median(Object beam)throws Exception {
        Line2D m=(Line2D)call(beam,"getMedian");return List.of(m.getX1(),m.getY1(),m.getX2(),m.getY2());
    }
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short","excess")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            List<?> stacks=(List<?>)call(system,"getStacks");for(Object s:stacks)call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",s);
            Object stack=stacks.get(0),measure=((List<?>)call(stack,"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt((Object ch)->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}).thenComparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).y;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();
            for(int i=0;i<chords.size();i++){Object ch=chords.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));int j=0;for(Object n:(Collection<?>)call(ch,"getMembers"))labels.put(n,"head"+i+"_"+j++);}
            List<Object> voices=new ArrayList<>((Collection<?>)call(measure,"getVoices"));for(int i=0;i<voices.size();i++)labels.put(voices.get(i),"voice"+i);
            Object old=voices.get(0),seed=call(old,"getFirstChord"),outside=call(old,"getLastChord"),offset=mode.equals("short")?make("math.Rational",1,8):make("math.Rational",5,4);
            call(outside,"setTimeOffset",offset);call(old,"checkDuration");call(seed,"setManual",true);
            Object group=call(seed,"getBeamGroup"),source=((List<?>)call(group,"getBeams")).get(0);labels.put(group,"source_group");labels.put(source,"source_beam");
            List<Object> relations=new ArrayList<>();for(Object r:(Collection<?>)call(sig,"edgeSet")){labels.put(r,"relation"+relations.size());relations.add(r);}
            Object rest=make("sig.inter.RestInter",glyph(355,240),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,call(system,"getFirstStaff"),0.),restChord=make("sig.inter.RestChordInter",.8);
            call(restChord,"setStaff",call(system,"getFirstStaff"));call(sig,"addVertex",rest);call(sig,"addVertex",restChord);call(restChord,"addMember",rest);call(measure,"addInter",restChord);
            labels.put(rest,"rest0");labels.put(restChord,"rest_chord0");chords.add(restChord);slots.put(restChord,null);
            List<Object> rests=List.of(restChord),baseline=new ArrayList<>();
            Object before=frame(sig,measure,stack,chords,voices,labels,slots,relations,rests,baseline,List.of(source));
            List<Object> groups=new ArrayList<>(),groupSpecs=new ArrayList<>();groups.add(group);
            List<Object> actual=new ArrayList<>();for(Object b:(List<?>)call(group,"getBeams")){if(!labels.containsKey(b))labels.put(b,"source_beam"+actual.size());actual.add(obj("label",labels.get(b),"kind",b.getClass().getSimpleName(),"median",median(b)));}
            groupSpecs.add(obj("label","source_group","actual_source",true,"beams",actual));
            if(args.length>2) {
                Map<?,?> fresh=new ObjectMapper().readValue(Path.of(args[2]).toFile(),Map.class);
                List<?> m=(List<?>)fresh.get("median");double[] raw=new double[4];for(int i=0;i<4;i++)raw[i]=((Number)m.get(i)).doubleValue();
                Object freshGroup=make("sig.inter.BeamGroupInter"),freshBeam=beam("BeamInter",raw);call(sig,"addVertex",freshGroup);call(sig,"addVertex",freshBeam);call(freshGroup,"addMember",freshBeam);
                labels.put(freshGroup,"fresh_source");labels.put(freshBeam,"fresh_source_beam");groups.add(freshGroup);
                groupSpecs.add(obj("label","fresh_source","original_fresh_geometry",true,"actual_source",false,"beams",List.of(obj("label","fresh_source_beam","kind","BeamInter","median",median(freshBeam)))));
            }
            String[][] kinds={{"BeamInter","BeamHookInter","BeamInter"},{"BeamInter","BeamInter"},{"BeamHookInter"},{"BeamInter","BeamInter"},{},{"SmallBeamInter"}};
            double[][][] medians={{{100,100,200,100},{100,110,200,110},{100,120,200,120}},{{100,120,200,120},{100,100,200,100}},{{100,110,200,110}},{{100,90,200,120},{120,160,180,155}},{},{{100,110,200,110}}};
            String[] names={"tie","reverse_tie","hook_only","sloped","empty","small_only"};
            for(int g=0;g<names.length;g++) {
                Object control=make("sig.inter.BeamGroupInter");call(sig,"addVertex",control);labels.put(control,names[g]);groups.add(control);
                List<Object> members=new ArrayList<>();for(int b=0;b<kinds[g].length;b++){Object v=beam(kinds[g][b],medians[g][b]);call(sig,"addVertex",v);call(control,"addMember",v);String label=names[g]+"_beam"+b;labels.put(v,label);members.add(obj("label",label,"kind",kinds[g][b],"median",median(v)));}
                groupSpecs.add(obj("label",names[g],"actual_source",false,"beams",members));
            }
            java.lang.reflect.Method nearest=c("sig.inter.BeamGroupInter").getDeclaredMethod("getNearestBeam",Point.class);nearest.setAccessible(true);
            List<Object> queries=new ArrayList<>();int[][] points={{100,90},{200,120},{99,90},{201,120},{150,110},{150,130},{300,250},{359,250},{450,200}};
            for(double slope:new double[]{0.,.02,-.03}) {
                call(sheet,"setSkew",make("sheet.Skew",slope,sheet));
                for(Object g:groups)for(int[] p:points){Object n=nearest.invoke(g,new Point(p[0],p[1]));Object selected=n==null?null:field(n,"sig.inter.BeamGroupInter$NearestBeam","beam"),dist=n==null?null:field(n,"sig.inter.BeamGroupInter$NearestBeam","dist");queries.add(obj("slope",call(call(system,"getSkew"),"getSlope"),"group",labels.get(g),"point",List.of(p[0],p[1]),"beam",labels.get(selected),"distance",dist));}
            }
            call(sheet,"setSkew",make("sheet.Skew",0.,sheet));
            Object after=frame(sig,measure,stack,chords,voices,labels,slots,relations,rests,baseline,List.of(source));
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(obj("bounds",List.of(355,240,8,20),"manual",false,"source_measure",true,"beam_link",false)),"groups",groupSpecs,"queries",queries,"state_before",before,"state_after",after));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual private getNearestBeam on actual retained/declared SIG groups and Sheet Skew; full-only, inclusive endpoints, strict member ties and null; timed owner/cache state held; BeamRest.added and full detection pending","cases",cases));
    }
}
