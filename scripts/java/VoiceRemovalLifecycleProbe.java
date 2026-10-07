// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Original Measure.removeInter/removeVoiceChord on timed actual owners. */
public class VoiceRemovalLifecycleProbe extends VoiceLifecycleProbe {
    static Object removalState(Object sig,Object measure,Object stack,List<Object> chords,
                               List<Object> retained,Map<Object,String> labels,Map<Object,Object> slots,
                               List<Object> relations) throws Exception {
        Map<?,?> all=(Map<?,?>)observe(sig,measure,stack,chords,retained,labels,slots,relations);
        Map<String,Object> result=new LinkedHashMap<>();
        for(String key:List.of("voices","measure_voices","measure_chords","measure_abnormal",
                               "expected","actual","excess","stack_slot_ids"))result.put(key,all.get(key));
        result.put("cached_chord_voices",chords.stream().map(ch->{try{return labels.get(call(ch,"getVoice"));}catch(Exception e){throw new RuntimeException(e);}}).toList());
        result.put("manual_seed",call(chords.get(0),"isManual"));
        return result;
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short","excess")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            Object stack=((List<?>)call(system,"getStacks")).get(0);
            call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",stack);
            Object measure=((List<?>)call(stack,"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));
            chords.sort(Comparator.comparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();
            for(int i=0;i<chords.size();i++) {
                Object ch=chords.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));
                int j=0;for(Object head:(Collection<?>)call(ch,"getMembers"))labels.put(head,"head"+i+"_"+j++);
            }
            Object seed=chords.get(0),old=call(seed,"getVoice");labels.put(old,"old_voice");
            List<Object> retained=new ArrayList<>(List.of(old)),relations=new ArrayList<>();
            for(Object rel:(Collection<?>)call(sig,"edgeSet")) {labels.put(rel,"relation"+relations.size());relations.add(rel);}
            call(seed,"setManual",true);
            Object last=chords.get(chords.size()-1);
            Object offset=mode.equals("short")?make("math.Rational",1,8):make("math.Rational",5,4);
            call(last,"setTimeOffset",offset);call(old,"checkDuration");
            List<Object> states=new ArrayList<>();
            states.add(obj("phase","timed","state",removalState(sig,measure,stack,chords,retained,labels,slots,relations)));
            for(int i=0;i<chords.size();i++) {
                // The original normal removal callback delegates to this public method.
                call(measure,"removeInter",chords.get(i));
                states.add(obj("phase","remove"+i,"state",removalState(sig,measure,stack,chords,retained,labels,slots,relations)));
            }
            call(measure,"removeInter",chords.get(chords.size()-1));
            states.add(obj("phase","repeat","state",removalState(sig,measure,stack,chords,retained,labels,slots,relations)));
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"states",states));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj(
            "scope","original timed Measure.removeInter/removeVoiceChord callback ownership only; SIG removal compared separately, not full graph equality",
            "cases",cases));
    }
}
