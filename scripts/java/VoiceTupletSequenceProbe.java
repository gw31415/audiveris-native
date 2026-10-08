// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.geom.Line2D;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class VoiceTupletSequenceProbe extends BeamRestClearProbe {
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short_mixed","excess_mixed","short_explicit_first","short_implicit_first","short_explicit_only")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            List<?> stacks=(List<?>)call(system,"getStacks");for(Object s:stacks)call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",s);
            Object stack=stacks.get(0),measure=((List<?>)call(stack,"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));chords.sort(Comparator.comparingInt((Object ch)->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}).thenComparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).y;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();for(int i=0;i<chords.size();i++){Object ch=chords.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));int j=0;for(Object n:(Collection<?>)call(ch,"getMembers"))labels.put(n,"head"+i+"_"+j++);}
            List<Object> voices=new ArrayList<>((Collection<?>)call(measure,"getVoices"));for(int i=0;i<voices.size();i++)labels.put(voices.get(i),"voice"+i);
            Object old=voices.get(0),seed=call(old,"getFirstChord"),last=call(old,"getLastChord"),offset=mode.startsWith("short")?make("math.Rational",1,8):make("math.Rational",5,4);call(last,"setTimeOffset",offset);call(old,"checkDuration");call(seed,"setManual",true);
            Object group=call(seed,"getBeamGroup"),source=((List<?>)call(group,"getBeams")).get(0);labels.put(group,"source_group");labels.put(source,"source_beam");
            List<Object> oldRelations=new ArrayList<>();for(Object r:(Collection<?>)call(sig,"edgeSet")){labels.put(r,"relation"+oldRelations.size());oldRelations.add(r);}

            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),tuplets=new ArrayList<>(),tupleRelations=new ArrayList<>(),declaredTuplets=new ArrayList<>(),declaredTupleLinks=new ArrayList<>();
            for(int i=0;i<3;i++){Object t=make("sig.inter.TupletInter",null,c("glyph.Shape").getField(i==1?"TUPLET_SIX":"TUPLET_THREE").get(null),.8);call(t,"setImplicit",i!=0);call(t,"setManual",i!=1);if(i==0)call(t,"setBounds",new java.awt.Rectangle(350,150,18,20));call(sig,"addVertex",t);labels.put(t,"tuplet"+i);tuplets.add(t);call(i==2?stack:measure,"addInter",t);declaredTuplets.add(obj("label","tuplet"+i,"shape",i==1?"TUPLET_SIX":"TUPLET_THREE","implicit",i!=0,"manual",i!=1,"unassigned",i==2,"bounds",i==0?List.of(350,150,18,20):null));}
            int[][] links=mode.endsWith("explicit_first")?new int[][]{{1,0},{1,2},{0,1}}:mode.endsWith("implicit_first")?new int[][]{{1,2},{1,0},{0,1}}:mode.endsWith("explicit_only")?new int[][]{{1,0},{0,0}}:new int[][]{{1,0},{1,2},{0,1},{2,2},{4,1}};for(int i=0;i<links.length;i++){Object edge=make("sig.relation.ChordTupletRelation");call(edge,"setManual",i%2==0);call(sig,"addEdge",chords.get(links[i][0]),tuplets.get(links[i][1]),edge);labels.put(edge,"tuple_link"+i);tupleRelations.add(edge);declaredTupleLinks.add(obj("source",labels.get(chords.get(links[i][0])),"target","tuplet"+links[i][1],"manual",i%2==0));}
            Object rhythm=make("sheet.rhythm.MeasureRhythm",measure);Method selected=c("sheet.rhythm.MeasureRhythm").getDeclaredMethod("getVoicesWithImplicit");selected.setAccessible(true);
            Object foreign=((Collection<?>)call(((List<?>)call(stacks.get(1),"getMeasures")).get(0),"getHeadChords")).iterator().next();labels.put(foreign,"foreign_chord");
            List<Object> phases=new ArrayList<>();for(String phase:List.of("before","allocated_empty","reset_null","purged_null","restored")){
                Object firstVoice=voices.get(0);List<Object> oldChords=new ArrayList<>((Collection<?>)call(firstVoice,"getChords"));
                if(phase.equals("allocated_empty")){for(Object ch:oldChords)call(firstVoice,"removeChord",ch);}
                if(phase.equals("reset_null"))call(firstVoice,"resetChords");if(phase.equals("purged_null"))call(measure,"purgeVoices");
                if(phase.equals("restored")){for(Object ch:chords)if(call(ch,"getVoice")==firstVoice)call(firstVoice,"addChord",ch);call(measure,"addVoice",firstVoice);}
                List<Object> queries=new ArrayList<>();for(Object voice:voices){List<Object> after=new ArrayList<>();for(Object ch:Arrays.asList(chords.get(0),chords.get(1),chords.get(3),chords.get(4),chords.get(7),foreign,null))after.add(obj("input",labels.get(ch),"after",labels.get(call(voice,"getChordAfter",ch))));Object ts=null;String error=null;try{ts=names((Collection<?>)call(voice,"getTuplets"),labels);}catch(Exception ex){Throwable t=ex;while(t.getCause()!=null)t=t.getCause();error=t.getClass().getSimpleName();}queries.add(obj("voice",labels.get(voice),"tuplets",ts,"tuplets_error",error,"last_with_tuplet",labels.get(call(voice,"getLastChordWithTuplet")),"after",after));}
                phases.add(obj("phase",phase,"selected",names((Collection<?>)selected.invoke(rhythm),labels),"queries",queries,"state",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));
                if(phase.equals("allocated_empty"))for(Object ch:oldChords)call(firstVoice,"addChord",ch);
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",declaredTuplets,"declared_tuple_links",declaredTupleLinks,"phases",phases));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual first-incident Tuplet selection and Voice getters/list reset; getLastChordWithTuplet sees any relation, getChordAfter absent/null returns first; current/retained Chord clocks unaffected","cases",cases));
    }
}
