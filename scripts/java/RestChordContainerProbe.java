// SPDX-License-Identifier: AGPL-3.0-or-later
// Original partial/last/extensive/shared rest-chord ownership removal.
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class RestChordContainerProbe extends FoundationOverlapProbe {
    static Object graphState(Object sig,List<Object> nodes,Map<Object,String> labels)throws Exception {
        List<Object> live=new ArrayList<>(),removed=new ArrayList<>(),rels=new ArrayList<>();
        for(Object n:nodes) {
            if((Boolean)call(n,"isRemoved")){removed.add(labels.get(n));continue;}
            Map<String,Object> row=obj("label",labels.get(n),"kind",n.getClass().getSimpleName(),"bounds",rect(call(n,"getBounds")),"grade",call(n,"getGrade"),"contextual_grade",call(n,"getContextualGrade"));
            if(n.getClass().getSimpleName().equals("RestChordInter")) {List<Object> ms=new ArrayList<>();for(Object m:(Collection<?>)call(n,"getMembers"))ms.add(labels.get(m));row.put("member_labels",ms);row.put("head_location",point(call(n,"getHeadLocation")));row.put("tail_location",point(call(n,"getTailLocation")));}
            else{row.put("shape",call(n,"getShape").toString());row.put("glyph",glyph(call(n,"getGlyph")));}
            live.add(row);
        }
        for(Object rel:(Collection<?>)call(sig,"edgeSet"))rels.add(obj("kind",rel.getClass().getSimpleName(),"source",labels.get(call(sig,"getEdgeSource",rel)),"target",labels.get(call(sig,"getEdgeTarget",rel))));
        return obj("live",live,"removed",removed,"relations",rels);
    }
    static Object state(Object sig,List<Object> nodes,Map<Object,String> labels)throws Exception {
        Object staff=call(nodes.get(0),"getStaff");
        java.lang.reflect.Field field=c("sheet.Staff").getDeclaredField("notes");field.setAccessible(true);
        List<Object> notes=new ArrayList<>();Object raw=field.get(staff);
        if(raw!=null)for(Object n:(Collection<?>)raw)if(labels.containsKey(n))notes.add(labels.get(n));
        List<Object> measures=new ArrayList<>();Set<Object> seen=Collections.newSetFromMap(new IdentityHashMap<>());
        for(Object n:nodes)if(n.getClass().getSimpleName().equals("RestChordInter")){
            Object m=call(n,"getMeasure");if(m!=null&&seen.add(m)){
                List<Object> registered=new ArrayList<>();for(Object ch:(Collection<?>)call(m,"getRestChords"))if(labels.containsKey(ch))registered.add(labels.get(ch));
                measures.add(obj("rest_chord_labels",registered));
            }
        }
        return obj("graph",graphState(sig,nodes,labels),"staff_note_labels",notes,"measure_registration",measures);
    }
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);List<Object> cases=new ArrayList<>();
        for(String name:List.of("partial-member","last-member","extensive-chord","shared-member")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet"),sys=((List<?>)call(sheet,"getSystems")).get(0),sig=call(sys,"getSig"),staff=call(sys,"getFirstStaff");
            for(Object n:new ArrayList<>((Collection<?>)call(sig,"vertexSet")))if(!(Boolean)call(n,"isRemoved"))call(n,"remove");
            List<Object> nodes=new ArrayList<>();Map<Object,String> labels=new IdentityHashMap<>();
            Object a=make("sig.inter.RestInter",physical(300,240,8,20,"VERTICAL",null),c("glyph.Shape").getField("QUARTER_REST").get(null),.4,staff,0.),b=make("sig.inter.RestInter",physical(340,242,8,20,"VERTICAL",null),c("glyph.Shape").getField("QUARTER_REST").get(null),.8,staff,0.);
            call(a,"setContextualGrade",.6);call(b,"setContextualGrade",.9);nodes.add(a);labels.put(a,"rest0");if(!name.equals("last-member")){nodes.add(b);labels.put(b,"rest1");}
            Object chord=make("sig.inter.RestChordInter",.8);call(chord,"setStaff",staff);nodes.add(chord);labels.put(chord,"chord");for(Object n:nodes)call(sig,"addVertex",n);call(chord,"addMember",a);if(!name.equals("last-member"))call(chord,"addMember",b);
            if(name.equals("shared-member")){Object other=make("sig.inter.RestChordInter",.8);call(other,"setStaff",staff);nodes.add(other);labels.put(other,"other");call(sig,"addVertex",other);call(other,"addMember",a);}
            Object initial=state(sig,nodes,labels);call(name.startsWith("extensive")||name.equals("shared-member")?chord:a,"remove");Object after=state(sig,nodes,labels);call(name.startsWith("extensive")||name.equals("shared-member")?chord:a,"remove");Object repeated=state(sig,nodes,labels);
            cases.add(obj("name",name,"initial",initial,"after",after,"repeat_is_idempotent",new ObjectMapper().valueToTree(after).equals(new ObjectMapper().valueToTree(repeated))));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj("scope","same four original rest-chord operations with actual Staff notes and Measure RestChord registration; pre-rhythm container projection, not voices/lyrics or full Part epilog","cases",cases));System.exit(0);
    }
}
