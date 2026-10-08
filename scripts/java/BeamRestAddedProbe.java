// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.awt.geom.Line2D;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Real addition callbacks before reset, including parallel relation identities. */
public class BeamRestAddedProbe extends BeamRestClearProbe {
    static class TracedRelation extends org.audiveris.omr.sig.relation.BeamRestRelation {
        final java.util.function.Consumer<String> observe;
        TracedRelation(java.util.function.Consumer<String> observe){this.observe=observe;}
        @Override public void added(org.jgrapht.event.GraphEdgeChangeEvent<org.audiveris.omr.sig.inter.Inter,org.audiveris.omr.sig.relation.Relation> event){
            observe.accept("entered");super.added(event);observe.accept("finished");
        }
    }
    static Object snapshot(Object sig,Object measure,Object stack,List<Object> chords,List<Object> voices,Map<Object,String> labels,Map<Object,Object> slots,List<Object> oldRelations,List<Object> rests,List<Object> baseline,List<Object> beams,List<Object> added)throws Exception {
        Map<String,Object> state=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,beams);
        List<Object> rows=new ArrayList<>();for(Object r:added)rows.add(obj("label",labels.get(r),"registered",call(sig,"containsEdge",r),"manual",call(r,"isManual"),"dy",call(r,"getDistance"),"portion",call(r,"getBeamPortion").toString()));
        state.put("added_relations",rows);state.put("live_added_order",names(((Collection<?>)call(sig,"edgeSet")).stream().filter(added::contains).toList(),labels));return state;
    }
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
            List<Object> beams=new ArrayList<>(List.of(source)),declaredBeams=new ArrayList<>();for(int i=0;i<2;i++){double y=i==0?187.5:186.5;Object b=make("sig.inter.BeamInter",make("sig.inter.AbstractBeamInter$Impacts",1.,1.,1.,1.,1.,1.),new Line2D.Double(288,y,470,y),4.);call(sig,"addVertex",b);call(group,"addMember",b);labels.put(b,"callback_beam"+i);beams.add(b);declaredBeams.add(obj("label",labels.get(b),"median",List.of(288.,y,470.,y),"height",4.));}
            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),declaredRests=new ArrayList<>();int[] centers={298,299,459,460,360,361};for(int i=0;i<centers.length;i++){int x=centers[i]-4;Object rest=make("sig.inter.RestInter",glyph(x,240),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,call(system,"getFirstStaff"),0.),chord=make("sig.inter.RestChordInter",.8);call(chord,"setStaff",call(system,"getFirstStaff"));call(sig,"addVertex",rest);call(sig,"addVertex",chord);call(chord,"addMember",rest);call(measure,"addInter",chord);labels.put(rest,"rest"+i);labels.put(chord,"rest_chord"+i);rests.add(chord);chords.add(chord);slots.put(chord,null);declaredRests.add(obj("bounds",List.of(x,240,8,20),"manual",false,"source_measure",true,"beam_link",false));}
            List<Object> actions=new ArrayList<>(),added=new ArrayList<>(),states=new ArrayList<>(),callbacks=new ArrayList<>();int[] bs={0,0,0,0,0,1,1},rs={0,1,2,3,0,4,5};boolean[] manual={true,false,false,false,false,false,true};double[] slopes={0,.02,-.03,0,0,0,0};
            states.add(obj("phase","before_add","state",snapshot(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,beams,added)));
            for(int i=0;i<rs.length;i++){
                call(sheet,"setSkew",make("sheet.Skew",slopes[i],sheet));final Object b=beams.get(bs[i]+1);final Object[] current=new Object[1];
                Object r=new TracedRelation(phase->{try{Object edge=current[0];List<Object> clocks=new ArrayList<>();for(Object ch:chords)clocks.add(obj("label",labels.get(ch),"manual",call(ch,"isManual"),"voice",labels.get(call(ch,"getVoice")),"slot_id",call(ch,"getSlot")==null?null:call(call(ch,"getSlot"),"getId"),"same_baseline_slot",call(ch,"getSlot")==slots.get(ch),"time_offset",rational(call(ch,"getTimeOffset"))));Object portion=call(edge,"getBeamPortion");callbacks.add(obj("phase",phase,"edge",labels.get(edge),"registered",call(sig,"containsEdge",edge),"manual",call(edge,"isManual"),"dy",call(edge,"getDistance"),"portion",portion==null?null:portion.toString(),"beam_abnormal",call(b,"isAbnormal"),"clocks",clocks));}catch(Exception e){throw new RuntimeException(e);}});
                current[0]=r;labels.put(r,"added"+i);call(r,"setManual",manual[i]);Object accepted=call(sig,"addEdge",b,((List<?>)call(rests.get(rs[i]),"getMembers")).get(0),r);added.add(r);actions.add(obj("beam",bs[i],"rest",rs[i],"manual",manual[i],"slope",slopes[i],"accepted",accepted));states.add(obj("phase","added"+i,"state",snapshot(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,beams,added)));
            }
            call(sheet,"setSkew",make("sheet.Skew",0.,sheet));for(String phase:List.of("clear","clear_again","measure_reset","rest_invalidate")){if(phase.startsWith("clear")){java.lang.reflect.Method m=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("clearInterleavedRests");m.setAccessible(true);m.invoke(make("sheet.rhythm.MeasureRhythm",measure));}if(phase.equals("measure_reset"))call(measure,"resetRhythm");if(phase.equals("rest_invalidate"))for(Object r:rests)call(r,"invalidateCache");states.add(obj("phase",phase,"state",snapshot(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,beams,added)));}
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"interline",call(call(sheet,"getScale"),"getInterline"),"declared_rests",declaredRests,"declared_beams",declaredBeams,"actions",actions,"callbacks",callbacks,"states",states));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual BeamRest.added and manual/parallel relation identity before clear/reset; ties-even dy, pixel gap boundaries and full-beam abnormal callback; old owner clocks/cache references held; plain lookup pending","cases",cases));
    }
}
