// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Genuine cached retired Voice lookup through NextInVoice refinement. */
public class VoiceRetiredNextProbe extends VoiceLifecycleProbe {
    static Object nextState(Object sig,Object measure,Object stack,List<Object> chords,
                            List<Object> retained,Map<Object,String> labels,Map<Object,Object> slots,
                            List<Object> relations,Object targetMeasure,Object targetVoice,
                            Object source,Object target,Object relation) throws Exception {
        return obj("source",observe(sig,measure,stack,chords,retained,labels,slots,relations),
            "target",voiceState(targetVoice,targetMeasure,labels),
            "source_ref_equal",call(source,"getVoice")==retained.get(0),
            "target_ref_equal",call(target,"getVoice")==targetVoice,
            "relation_registered",call(sig,"containsEdge",relation),"relation_manual",call(relation,"isManual"));
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for(String mode:List.of("short","excess")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            List<?> stacks=(List<?>)call(system,"getStacks");
            for(Object s:stacks)call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",s);
            Object stack=stacks.get(0);
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
            Object targetMeasure=((List<?>)call(stacks.get(1),"getMeasures")).get(0);
            List<Object> nextChords=new ArrayList<>((Collection<?>)call(targetMeasure,"getHeadChords"));
            nextChords.sort(Comparator.comparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
            for(int i=0;i<nextChords.size();i++) {
                Object ch=nextChords.get(i);labels.put(ch,"chord"+(i+4));
                int j=0;for(Object head:(Collection<?>)call(ch,"getMembers"))labels.put(head,"head"+(i+4)+"_"+j++);
            }
            Object target=nextChords.get(0),targetVoice=call(target,"getVoice");labels.put(targetVoice,"target_voice");
            call(old,"setId",7);call(old,"resetChords");
            Object sourceChord=chords.get(chords.size()-1),relation=make("sig.relation.NextInVoiceRelation");
            call(relation,"setManual",true);call(sig,"addEdge",sourceChord,target,relation);
            labels.put(relation,"relation"+relations.size());relations.add(relation);
            List<Object> states=new ArrayList<>();
            for(String phase:List.of("before","refined","repeat","preferred")) {
                if(phase.equals("preferred"))call(target,"setPreferredVoiceId",2);
                if(!phase.equals("before"))refine(system);
                states.add(obj("phase",phase,"state",nextState(sig,measure,stack,chords,retained,labels,slots,relations,
                    targetMeasure,targetVoice,sourceChord,target,relation)));
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"states",states));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj(
            "scope","original retired empty Voice7 cached by previous-Measure Chord; actual NextInVoice then preferred-ID refinement, not automatic link recognition",
            "cases",cases));
    }
}
