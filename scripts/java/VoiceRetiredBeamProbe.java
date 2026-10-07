// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Actual BeamGroup predecessor with purged and restored Voice ownership. */
public class VoiceRetiredBeamProbe extends VoiceRetiredTieProbe {
    static Object frame(Object sig,Object measure,Object stack,List<Object> chords,List<Object> refs,
                        Map<Object,String> labels,Map<Object,Object> slots,List<Object> relations,
                        Object targetMeasure,List<Object> targetRefs,Object group,Object predecessor,Object target) throws Exception {
        List<Object> targetRows=new ArrayList<>(),groupRows=new ArrayList<>();
        for(Object v:targetRefs)targetRows.add(voiceState(v,targetMeasure,labels));
        for(Object ch:(List<?>)call(group,"getChords"))groupRows.add(obj("chord",labels.get(ch),
            "previous_measure",call(ch,"getMeasure")==measure,"voice",labels.get(call(ch,"getVoice"))));
        Object groupVoice=call(group,"getVoice");List<Object> cachedIds=new ArrayList<>();
        for(Object ch:chords){Object v=call(ch,"getVoice");cachedIds.add(v==null?null:call(v,"getId"));}
        return obj("group_voice",labels.get(groupVoice),"group_voice_id",groupVoice==null?null:call(groupVoice,"getId"),"cached_chord_voice_ids",cachedIds,"source",observe(sig,measure,stack,chords,refs,labels,slots,relations),"target",targetRows,
            "group_chords",groupRows,"group_contains_previous",((Collection<?>)call(group,"getMeasures")).contains(measure),
            "same_target_group",call(target,"getBeamGroup")==group,"predecessor",labels.get(predecessor),
            "predecessor_ref_old",call(predecessor,"getVoice")==refs.get(0),"tied_id",tied(targetRefs.get(0)));
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
            Object stack=stacks.get(0),measure=((List<?>)call(stack,"getMeasures")).get(0);
            Object targetMeasure=((List<?>)call(stacks.get(1),"getMeasures")).get(0);
            List<?> sourceVoices=(List<?>)call(measure,"getVoices"),targetVoices=(List<?>)call(targetMeasure,"getVoices");
            Object old=sourceVoices.get(0),support=sourceVoices.get(1),seed=call(old,"getFirstChord"),last=call(old,"getLastChord");
            Object targetVoice=targetVoices.get(0),target=call(targetVoice,"getFirstChord"),supportLast=call(support,"getLastChord");
            Map<Object,String> labels=new IdentityHashMap<>();Map<Object,Object> slots=new IdentityHashMap<>();
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));
            Comparator<Object> position=Comparator.comparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}});
            position=position.thenComparingInt(ch->{try{return ((java.awt.Rectangle)call(ch,"getBounds")).y;}catch(Exception e){throw new RuntimeException(e);}});
            chords.sort(position);
            List<Object> all=new ArrayList<>(chords),next=new ArrayList<>((Collection<?>)call(targetMeasure,"getHeadChords"));next.sort(position);all.addAll(next);
            List<Object> geometry=new ArrayList<>();
            for(int i=0;i<all.size();i++) {Object ch=all.get(i);labels.put(ch,"chord"+i);slots.put(ch,call(ch,"getSlot"));
                geometry.add(obj("label",labels.get(ch),"bounds",rect(call(ch,"getBounds"))));
                int j=0;for(Object head:(Collection<?>)call(ch,"getMembers"))labels.put(head,"head"+i+"_"+j++);}
            labels.put(old,"old_voice");labels.put(support,"support_voice");labels.put(targetVoice,"target_voice");labels.put(targetVoices.get(1),"target_lower_voice");
            List<Object> refs=new ArrayList<>(List.of(old,support)),targetRefs=List.of(targetVoice,targetVoices.get(1)),relations=new ArrayList<>();
            for(Object relation:(Collection<?>)call(sig,"edgeSet")){labels.put(relation,"relation"+relations.size());relations.add(relation);}
            call(seed,"setManual",true);Object offset=mode.equals("short")?make("math.Rational",1,8):make("math.Rational",5,4);
            call(last,"setTimeOffset",offset);call(old,"checkDuration");call(old,"setId",7);call(old,"resetChords");
            tie(sig,supportLast,target,labels,relations);
            Object beam=((List<?>)call(seed,"getBeams")).get(0),relation=make("sig.relation.BeamStemRelation");call(relation,"setManual",true);
            call(relation,"setBeamPortion",c("sig.relation.BeamPortion").getField("LEFT").get(null));call(sig,"addEdge",beam,call(target,"getStem"),relation);
            labels.put(relation,"relation"+relations.size());relations.add(relation);Object group=call(target,"getBeamGroup");
            List<?> groupChords=(List<?>)call(group,"getChords");Object predecessor=groupChords.get(groupChords.indexOf(target)-1);
            List<Object> states=new ArrayList<>();
            for(String phase:List.of("before","refined","reused","reuse_refined","restored_live","live_refined","emptied_live","repurged","next","preferred","null","restored","measure_reset")) {
                switch(phase) {
                    case "reused" -> {Object fresh=make("sheet.rhythm.Voice",seed,measure);call(measure,"addVoice",fresh);call(fresh,"setId",7);labels.put(fresh,"new_voice");refs.add(fresh);}
                    case "restored_live" -> {call(old,"addChord",predecessor);call(measure,"addVoice",old);}
                    case "emptied_live" -> call(old,"resetChords");
                    case "next" -> {Object r=make("sig.relation.NextInVoiceRelation");call(r,"setManual",true);call(sig,"addEdge",supportLast,target,r);labels.put(r,"relation"+relations.size());relations.add(r);}
                    case "preferred" -> call(target,"setPreferredVoiceId",1);
                    case "null" -> call(predecessor,"justAssignVoice",(Object)null);
                    case "restored" -> call(predecessor,"justAssignVoice",old);
                    case "measure_reset" -> call(measure,"resetRhythm");
                }
                if(List.of("refined","reuse_refined","live_refined","repurged","next","preferred").contains(phase))refine(system);
                states.add(obj("phase",phase,"state",frame(sig,measure,stack,chords,refs,labels,slots,relations,targetMeasure,targetRefs,group,predecessor,target)));
            }
            String failure=null;try{refine(system);}catch(Exception e){Throwable t=e;while(t.getCause()!=null)t=t.getCause();failure=t.getClass().getSimpleName();}
            cases.add(obj("name",mode,"declared_last_offset",offset.toString(),"geometry",geometry,"seed",labels.get(seed),"last",labels.get(last),"support_last",labels.get(supportLast),"target",labels.get(target),"beam_bounds",rect(call(beam,"getBounds")),"states",states,"reset_refine_failure",failure));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","actual owned BeamGroup predecessor/previous Measure; declared crossing BeamStem and tie; retired/reused/restored live owners before Next/preferred and genuine reset; no automatic cross-bar admission or Java exception rollback parity","cases",cases));
    }
}
