// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Original retained Voice objects, independent of native reports. */
public class VoiceLifecycleProbe extends HeadStemPreLinkProbe {
    static Object voiceState(Object voice, Object measure, Map<Object,String> labels) throws Exception {
        List<Object> slots=new ArrayList<>();
        Map<?,?> table=(Map<?,?>)field(voice,"sheet.rhythm.Voice","slots");
        if(table!=null)for(var entry:table.entrySet()) {
            Object info=entry.getValue();
            slots.add(obj("id",entry.getKey(),"chord",labels.get(field(info,"sheet.rhythm.SlotVoice","chord")),
                "status",field(info,"sheet.rhythm.SlotVoice","status").toString()));
        }
        return obj("label",labels.get(voice),"id",call(voice,"getId"),
            "measure_registered",((Collection<?>)call(measure,"getVoices")).contains(voice),
            "same_measure",call(voice,"getMeasure")==measure,
            "chords",names((Collection<?>)call(voice,"getChords"),labels),"slots",slots,
            "termination",rational(call(voice,"getTermination")),
            "excess",rational(field(voice,"sheet.rhythm.Voice","excess")));
    }
    static Object observe(Object sig, Object measure, Object stack, List<Object> chords,
                          List<Object> retained, Map<Object,String> labels, Map<Object,Object> slots,
                          List<Object> relations) throws Exception {
        List<Object> chordRows=new ArrayList<>(),voiceRows=new ArrayList<>(),relationRows=new ArrayList<>();
        for(Object ch:chords)chordRows.add(obj("label",labels.get(ch),"registered",call(sig,"containsVertex",ch),
            "same_measure",call(ch,"getMeasure")==measure,"manual",call(ch,"isManual"),
            "members",names((Collection<?>)call(ch,"getMembers"),labels),
            "voice",labels.get(call(ch,"getVoice")),"same_baseline_slot",call(ch,"getSlot")==slots.get(ch),
            "slot_id",call(ch,"getSlot")==null?null:call(call(ch,"getSlot"),"getId"),
            "time_offset",rational(call(ch,"getTimeOffset")),"duration",rational(call(ch,"getDuration")),
            "end_time",rational(call(ch,"getEndTime"))));
        for(Object voice:retained)voiceRows.add(voiceState(voice,measure,labels));
        for(Object rel:relations)relationRows.add(obj("label",labels.get(rel),"registered",call(sig,"containsEdge",rel),
            "manual",call(rel,"isManual")));
        return obj("chords",chordRows,"voices",voiceRows,
            "measure_voices",names((Collection<?>)call(measure,"getVoices"),labels),
            "measure_chords",names((Collection<?>)call(measure,"getHeadChords"),labels),
            "measure_abnormal",call(measure,"isAbnormal"),"expected",rational(call(stack,"getExpectedDuration")),
            "actual",rational(call(stack,"getActualDuration")),"excess",rational(call(stack,"getExcess")),
            "stack_slot_ids",((Collection<?>)call(stack,"getSlots")).stream().map(s->{try{return call(s,"getId");}catch(Exception e){throw new RuntimeException(e);}}).toList(),
            "relations",relationRows);
    }
    static void refine(Object system) throws Exception {
        c("sheet.rhythm.Voices").getMethod("refineSystem",c("sheet.SystemInfo")).invoke(null,system);
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
            states.add(obj("phase","timed","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            call(old,"resetChords");
            states.add(obj("phase","emptied","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            refine(system);
            states.add(obj("phase","purged","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            refine(system);
            states.add(obj("phase","purged_again","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            call(measure,"addVoice",old);
            states.add(obj("phase","restored_old","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            refine(system);
            states.add(obj("phase","repurged","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            Object fresh=make("sheet.rhythm.Voice",seed,measure);call(measure,"addVoice",fresh);
            labels.put(fresh,"new_voice");retained.add(fresh);
            states.add(obj("phase","reused","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            refine(system);
            states.add(obj("phase","refined_reused","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            call(measure,"resetRhythm");
            states.add(obj("phase","measure_reset","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            call(measure,"resetRhythm");
            states.add(obj("phase","measure_reset_again","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            call(stack,"resetRhythm");
            states.add(obj("phase","stack_reset","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            Object rebuilt=make("sheet.rhythm.Voice",seed,measure);call(measure,"addVoice",rebuilt);
            labels.put(rebuilt,"rebuilt_voice");retained.add(rebuilt);
            states.add(obj("phase","rebuilt","state",observe(sig,measure,stack,chords,retained,labels,slots,relations)));
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"states",states));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj(
            "original_source","9e1e55cd2746037d059345881c53e6a6754bffbd",
            "scope","original actual retained Voice purge, musical-number reuse and legitimate Measure/stack clock reset; declared last time and manual Chord, no native output input or automatic image/controller history claim",
            "cases",cases));System.exit(0);
    }
}
