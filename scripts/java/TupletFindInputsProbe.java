// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.util.List;
import java.awt.*;
import java.lang.reflect.Method;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletFindInputsProbe extends BeamRestClearProbe {
    static void setCached(Object object,String name,Object value)throws Exception {for(Class<?> type=object.getClass();type!=null;type=type.getSuperclass())try{java.lang.reflect.Field f=type.getDeclaredField(name);f.setAccessible(true);f.set(object,value);return;}catch(NoSuchFieldException ex){}throw new NoSuchFieldException(name);}
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

            List<Object> rests=new ArrayList<>(),baseline=new ArrayList<>(),phases=new ArrayList<>();Object generator=make("sheet.rhythm.TupletGenerator",measure);
            Object before=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));List<Object> oldChords=new ArrayList<>((Collection<?>)call(old,"getChords"));for(Object ch:oldChords)call(old,"removeChord",ch);Object allocatedEmpty=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source)),allocatedSum=call(old,"getDurationSansTuplet");boolean allocatedNull=field(old,"sheet.rhythm.Voice","chords")==null;for(Object ch:oldChords)call(old,"addChord",ch);call(old,"resetChords");String emptyError=null;try{call(generator,"findImplicitTuplets");}catch(Exception ex){Throwable cause=ex;while(cause.getCause()!=null)cause=cause.getCause();emptyError=cause.getClass().getSimpleName();}
            Object empty=frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source));boolean emptyNull=field(old,"sheet.rhythm.Voice","chords")==null;Object emptySum=null;String emptySumError=null;try{emptySum=call(old,"getDurationSansTuplet");}catch(Exception ex){Throwable cause=ex;while(cause.getCause()!=null)cause=cause.getCause();emptySumError=cause.getClass().getSimpleName();}
            for(Object ch:chords)if(call(ch,"getVoice")==old)call(old,"addChord",ch);
            for(Object voice:voices)call(call(voice,"getFirstChord"),"setTimeOffset",(Object)null);call(stack,"setExpectedDuration",(Object)null);
            Object rest=make("sig.inter.RestInter",glyph(500,230),c("glyph.Shape").getField("WHOLE_REST").get(null),.8,call(system,"getFirstStaff"),0.);Object chord=make("sig.inter.RestChordInter",.8);call(chord,"setStaff",call(system,"getFirstStaff"));call(sig,"addVertex",rest);call(sig,"addVertex",chord);call(chord,"addMember",rest);call(measure,"addInter",chord);labels.put(rest,"rest0");labels.put(chord,"rest_chord0");rests.add(chord);chords.add(chord);slots.put(chord,null);
            Object voice=c("sheet.rhythm.Voice").getMethod("createMeasureRestVoice",c("sig.inter.RestChordInter"),c("sheet.rhythm.Measure")).invoke(null,chord,measure);call(measure,"addVoice",voice);labels.put(voice,"voice2");voices.add(voice);
            List<Object> rows=new ArrayList<>();for(String phase:List.of("rest_created","rest_chords_reset")){if(phase.equals("rest_chords_reset"))call(voice,"resetChords");Collection<?> found=(Collection<?>)call(generator,"findImplicitTuplets");rows.add(obj("phase",phase,"id",call(voice,"getId"),"measure_rest",call(voice,"isMeasureRest"),"chords_null",field(voice,"sheet.rhythm.Voice","chords")==null,"chords",names((Collection<?>)call(voice,"getChords"),labels),"sum",rational(call(voice,"getDurationSansTuplet")),"same_rest_chord",call(voice,"getFirstChord")==chord,"rest_chord_voice",call(chord,"getVoice")==voice,"registered",((Collection<?>)call(measure,"getVoices")).contains(voice),"found_count",found.size(),"state",frame(sig,measure,stack,chords,voices,labels,slots,oldRelations,rests,baseline,List.of(source))));}
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"declared_rests",List.of(),"declared_beams",List.of(),"declared_links",List.of(),"declared_tuplets",List.of(),"declared_tuple_links",List.of(),"before",before,"allocated_empty",allocatedEmpty,"allocated_empty_sum",rational(allocatedSum),"allocated_chords_null",allocatedNull,"empty_chords_null",emptyNull,"empty_error",emptyError,"empty_sum",rational(emptySum),"empty_sum_error",emptySumError,"empty",empty,"rest_declaration",obj("shape","WHOLE_REST","bounds",List.of(500,230,8,20),"source_measure",true,"beam_link",false),"phases",rows));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual registered ordinary Voice resetChords find failure versus actual createMeasureRestVoice(real registered WHOLE_REST/RestChord) empty-list/null-sum bypass before expected; Chord/backrefs/old slots/end/excess unchanged, no fake missing Voice","cases",cases));
    }
}
