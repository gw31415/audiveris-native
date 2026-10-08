// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Pre-reset relation removal on real RestChord/Measure and retained clocks. */
public class BeamRestClearProbe extends VoiceLifecycleProbe {
    static Object glyph(int x,int y) throws Exception {
        byte[] pixels=new byte[8*20];
        Object image=Class.forName("ij.process.ByteProcessor").getConstructor(int.class,int.class,byte[].class).newInstance(8,20,pixels);
        Object factory=make("run.RunTableFactory",c("run.Orientation").getField("VERTICAL").get(null));
        return make("glyph.Glyph",x,y,call(factory,"createTable",image));
    }
    static Map<String,Object> frame(Object sig,Object measure,Object stack,List<Object> chords,
        List<Object> voices,Map<Object,String> labels,Map<Object,Object> slots,List<Object> relations,
        List<Object> rests,List<Object> baseline,List<Object> beams) throws Exception {
        List<Object> restRows=new ArrayList<>(),beamRows=new ArrayList<>();
        for(int i=0;i<rests.size();i++) {
            Object rest=rests.get(i),list=call(rest,"getBeams");
            if(baseline.size()==i)baseline.add(list);
            restRows.add(obj("label",labels.get(rest),"beams",names((Collection<?>)list,labels),
                "same_baseline_beams",list==baseline.get(i),"group",labels.get(call(rest,"getBeamGroup")),
                "same_measure",call(rest,"getMeasure")==measure,"voice",labels.get(call(rest,"getVoice")),
                "members",names((Collection<?>)call(rest,"getMembers"),labels)));
        }
        for(Object beam:beams)beamRows.add(obj("label",labels.get(beam),"abnormal",call(beam,"isAbnormal")));
        return obj("owned",observe(sig,measure,stack,chords,voices,labels,slots,relations),
            "rests",restRows,"beams",beamRows,"measure_groups",names((Collection<?>)call(measure,"getBeamGroups"),labels),
            "measure_rests",names((Collection<?>)call(measure,"getRestChords"),labels));
    }
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short_auto","short_manual","short_mixed","excess_auto","excess_manual","excess_mixed")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            List<?> stacks=(List<?>)call(system,"getStacks");
            for(Object s:stacks)call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",s);
            Object stack=stacks.get(0),measure=((List<?>)call(stack,"getMeasures")).get(0),targetMeasure=((List<?>)call(stacks.get(1),"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));
            chords.sort(Comparator.comparingInt((Object ch)->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}).thenComparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).y;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();
            for(int i=0;i<chords.size();i++){Object ch=chords.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));int j=0;for(Object n:(Collection<?>)call(ch,"getMembers"))labels.put(n,"head"+i+"_"+j++);}
            List<Object> voices=new ArrayList<>((Collection<?>)call(measure,"getVoices"));for(int i=0;i<voices.size();i++)labels.put(voices.get(i),"voice"+i);
            Object old=voices.get(0),last=call(old,"getLastChord"),offset=make("math.Rational",mode.startsWith("short")?1:5,mode.startsWith("short")?8:4);
            call(last,"setTimeOffset",offset);call(old,"checkDuration");call(call(old,"getFirstChord"),"setManual",true);
            Object group=call(call(old,"getFirstChord"),"getBeamGroup"),beam=((List<?>)call(group,"getBeams")).get(0);
            labels.put(group,"source_group");labels.put(beam,"source_beam");
            List<Object> relations=new ArrayList<>();for(Object r:(Collection<?>)call(sig,"edgeSet")){labels.put(r,"relation"+relations.size());relations.add(r);}
            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),declared=new ArrayList<>();
            for(int i=0;i<3;i++) {
                Object m=i<2?measure:targetMeasure;
                java.awt.geom.Line2D median=(java.awt.geom.Line2D)call(beam,"getMedian");
                int x=i<2?(int)Math.rint((median.getX1()+median.getX2())/2)+(i==0?-20:20):900,y=240;
                Object rest=make("sig.inter.RestInter",glyph(x,y),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,call(system,"getFirstStaff"),0.);
                Object chord=make("sig.inter.RestChordInter",.8);call(chord,"setStaff",call(system,"getFirstStaff"));call(sig,"addVertex",rest);call(sig,"addVertex",chord);call(chord,"addMember",rest);call(m,"addInter",chord);
                labels.put(rest,"rest"+i);labels.put(chord,"rest_chord"+i);rests.add(chord);
                if(i<2){chords.add(chord);slots.put(chord,null);}
                boolean manual=i<2&&(mode.endsWith("manual")||(mode.endsWith("mixed")&&i==1));
                if(i<2){Object relation=make("sig.relation.BeamRestRelation");call(relation,"setManual",manual);call(sig,"addEdge",beam,rest,relation);labels.put(relation,"beam_rest"+i);relations.add(relation);}
                declared.add(obj("bounds",List.of(x,y,8,20),"manual",manual,"source_measure",i<2));
            }
            List<Object> states=new ArrayList<>();Object rhythm=make("sheet.rhythm.MeasureRhythm",measure);
            for(String phase:List.of("before_clear","cleared","cleared_again","measure_reset","rest_invalidate")) {
                if(phase.startsWith("cleared")){java.lang.reflect.Method method=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("clearInterleavedRests");method.setAccessible(true);method.invoke(rhythm);}
                if(phase.equals("measure_reset"))call(measure,"resetRhythm");
                if(phase.equals("rest_invalidate"))for(int i=0;i<2;i++)call(rests.get(i),"invalidateCache");
                states.add(obj("phase",phase,"state",frame(sig,measure,stack,chords,voices,labels,slots,relations,rests,baseline,List.of(beam))));
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",declared,"states",states));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual private clearInterleavedRests before reset; manual/automatic relation identity, owned source/target rests, retained Voice clocks and cached beam list through reset/invalidate; detection and normal rhythm connection pending","cases",cases));
    }
}
