// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import java.lang.reflect.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Actual retired incoming tie, null fallback and distinct reused Voice number. */
public class VoiceRetiredTieProbe extends VoiceLifecycleProbe {
    static Object tied(Object voice) throws Exception {
        Class<?> adapter=c("sheet.rhythm.Voices$SlurAdapter");
        Object a=Proxy.newProxyInstance(adapter.getClassLoader(),new Class<?>[]{adapter},(proxy,method,args)->args[0]);
        Method m=c("sheet.rhythm.Voices").getDeclaredMethod("getTiedId",c("sheet.rhythm.Voice"),adapter);
        m.setAccessible(true);return m.invoke(null,voice,a);
    }
    static void tie(Object sig,Object from,Object to,Map<Object,String> labels,List<Object> relations) throws Exception {
        Object slur=make("sig.inter.SlurInter",true,.8);call(slur,"setManual",true);call(sig,"addVertex",slur);
        for(String side:List.of("LEFT","RIGHT")) {
            Object chord=side.equals("LEFT")?from:to;
            Object relation=make("sig.relation.SlurHeadRelation",c("util.HorizontalSide").getField(side).get(null));
            call(relation,"setManual",true);call(sig,"addEdge",slur,((List<?>)call(chord,"getNotes")).get(0),relation);
            labels.put(relation,"relation"+relations.size());relations.add(relation);
        }
        call(slur,"setTie",true);
    }
    static Object snapshot(Object sig,Object measure,Object stack,List<Object> chords,List<Object> retained,
                           Map<Object,String> labels,Map<Object,Object> slots,List<Object> relations,
                           Object targetMeasure,Object targetVoice,Object source,Object target) throws Exception {
        Object head=((List<?>)call(source,"getNotes")).get(0),fallback=chords.get(0);
        return obj("source",observe(sig,measure,stack,chords,retained,labels,slots,relations),
            "target",voiceState(targetVoice,targetMeasure,labels),"tied_id",tied(targetVoice),
            "source_ref_equal",call(source,"getVoice")==retained.get(0),
            "head_ref_equal",call(head,"getVoice")==call(source,"getVoice"),
            "fallback_ref_equal",retained.size()>1&&call(fallback,"getVoice")==retained.get(1),
            "target_ref_equal",call(target,"getVoice")==targetVoice);
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
            Object source=chords.get(chords.size()-1);
            tie(sig,source,target,labels,relations);
            List<Object> states=new ArrayList<>();
            for(String phase:List.of("before","refined","reused","null","null_refined","restored",
                                     "restored_refined","next","preferred","measure_reset")) {
                switch(phase) {
                    case "reused" -> {
                        Object fresh=make("sheet.rhythm.Voice",seed,measure);call(measure,"addVoice",fresh);call(fresh,"setId",7);
                        labels.put(fresh,"new_voice");retained.add(fresh);tie(sig,seed,target,labels,relations);
                    }
                    case "null" -> call(source,"justAssignVoice",(Object)null);
                    case "restored" -> call(source,"justAssignVoice",old);
                    case "next" -> {
                        Object relation=make("sig.relation.NextInVoiceRelation");call(relation,"setManual",true);
                        call(sig,"addEdge",seed,target,relation);labels.put(relation,"relation"+relations.size());relations.add(relation);
                    }
                    case "preferred" -> call(target,"setPreferredVoiceId",2);
                    case "measure_reset" -> call(measure,"resetRhythm");
                }
                if(List.of("refined","null_refined","restored_refined","next","preferred").contains(phase))refine(system);
                states.add(obj("phase",phase,"state",snapshot(sig,measure,stack,chords,retained,labels,slots,relations,
                    targetMeasure,targetVoice,source,target)));
            }
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"states",states));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj(
            "scope","original retired incoming tie7, same-number fresh7, null first-source fallback, restored cached old reference, NextInVoice/preferred order and legitimate Measure reset; declared ties, no automatic curve or BeamGroup claim",
            "cases",cases));
    }
}
