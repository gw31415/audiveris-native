// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletGettersProbe extends BeamRestClearProbe {
    static void declaredField(Object object,String name,Object value)throws Exception {for(Class<?> type=object.getClass();type!=null;type=type.getSuperclass())try{java.lang.reflect.Field f=type.getDeclaredField(name);f.setAccessible(true);f.set(object,value);return;}catch(NoSuchFieldException ex){}throw new NoSuchFieldException(name);}
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

            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),tuplets=new ArrayList<>(),relations=new ArrayList<>(),declaredTuplets=new ArrayList<>();Object staff=call(system,"getFirstStaff");
            for(int i=0;i<3;i++){Object t=make("sig.inter.TupletInter",null,c("glyph.Shape").getField(i==1?"TUPLET_SIX":"TUPLET_THREE").get(null),.8);call(t,"setImplicit",true);call(t,"setManual",i!=0);call(sig,"addVertex",t);call(i==2?stack:measure,"addInter",t);labels.put(t,"tuplet"+i);tuplets.add(t);declaredTuplets.add(obj("label","tuplet"+i,"shape",i==1?"TUPLET_SIX":"TUPLET_THREE","implicit",true,"manual",i!=0,"unassigned",i==2,"bounds",null));}
            Map<Object,Object> firstFactors=new IdentityHashMap<>(),firstBases=new IdentityHashMap<>(),firstStaffs=new IdentityHashMap<>();List<Object> phases=new ArrayList<>();
            for(String phase:List.of("unlinked","linked","declared_shape_changed","note_staff_null","tuple_staff_cleared","notes_restored","first_link_removed","late_faster_link","measure_reset","fresh_number_reused")){
                if(phase.equals("linked")){int[][] links={{0,0},{1,0},{4,1},{0,1}};for(int i=0;i<links.length;i++){Object edge=make("sig.relation.ChordTupletRelation");call(edge,"setManual",i%2==0);call(sig,"addEdge",chords.get(links[i][0]),tuplets.get(links[i][1]),edge);labels.put(edge,"tuple_link"+relations.size());relations.add(edge);}}
                if(phase.equals("declared_shape_changed")){declaredField(tuplets.get(0),"shape",c("glyph.Shape").getField("TUPLET_SIX").get(null));declaredField(tuplets.get(1),"shape",c("glyph.Shape").getField("TUPLET_THREE").get(null));}
                if(phase.equals("note_staff_null")||phase.equals("notes_restored")){for(int index:List.of(0,1)){Object ch=chords.get(index);call(ch,"setStaff",(Object)null);for(Object note:(Collection<?>)call(ch,"getMembers"))call(note,"setStaff",phase.equals("note_staff_null")?null:staff);}}
                if(phase.equals("tuple_staff_cleared"))call(tuplets.get(0),"setStaff",(Object)null);
                if(phase.equals("first_link_removed"))call(sig,"removeEdge",relations.get(0));
                if(phase.equals("late_faster_link")){int[][] links={{2,1},{0,2}};for(int[] link:links){Object edge=make("sig.relation.ChordTupletRelation");call(edge,"setManual",true);call(sig,"addEdge",chords.get(link[0]),tuplets.get(link[1]),edge);labels.put(edge,"tuple_link"+relations.size());relations.add(edge);}}
                if(phase.equals("measure_reset"))call(measure,"resetRhythm");
                if(phase.equals("fresh_number_reused")){Object v=make("sheet.rhythm.Voice",chords.get(0),measure);call(measure,"addVoice",v);labels.put(v,"voice"+voices.size());voices.add(v);}
                List<Object> rows=new ArrayList<>();for(Object t:tuplets){Object factor=call(t,"getDurationFactor"),base=call(t,"getBaseDuration"),st=call(t,"getStaff"),part=call(t,"getPart"),voice=call(t,"getVoice");firstFactors.putIfAbsent(t,factor);if(base!=null)firstBases.putIfAbsent(t,base);if(st!=null)firstStaffs.putIfAbsent(t,st);rows.add(obj("label",labels.get(t),"shape",call(t,"getShape").toString(),"manual",call(t,"isManual"),"factor",factor.toString(),"same_factor",factor==firstFactors.get(t),"base",rational(base),"same_base",base==firstBases.get(t),"staff",st==null?null:call(st,"getId"),"same_staff",st==firstStaffs.get(t),"part",part==null?null:call(part,"getId"),"voice",labels.get(voice),"chords",names((Collection<?>)call(t,"getChords"),labels)));}
                List<Object> rs=new ArrayList<>();for(Object edge:relations)rs.add(obj("label",labels.get(edge),"registered",call(sig,"containsEdge",edge),"manual",call(edge,"isManual")));phases.add(obj("phase",phase,"getters",rows,"relations",rs,"state",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",declaredTuplets,"declared_tuple_links",List.of(),"phases",phases));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","original Tuplet lazy factor/base/Staff/Part retention (shape variation is declared private field control, no original setShape API), first live incoming Voice/null after genuine reset and number reuse; same actual objects/manual links; null note/Chord Staff does not relocate Measure; no archive/normal XML admission","cases",cases));
    }
}
