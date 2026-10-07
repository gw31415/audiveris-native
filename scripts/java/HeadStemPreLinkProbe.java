// SPDX-License-Identifier: AGPL-3.0-or-later
import java.awt.Rectangle;
import java.lang.reflect.*;
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;

/** Observe the original controller's preLink task list on a real timed owner. */
public class HeadStemPreLinkProbe {
    static Class<?> c(String name) throws Exception {
        return Class.forName("org.audiveris.omr." + name);
    }
    static Object call(Object object, String name, Object... args) throws Exception {
        for (Method method : object.getClass().getMethods()) {
            if (!method.getName().equals(name) || method.getParameterCount() != args.length) continue;
            try { return method.invoke(object, args); }
            catch (IllegalArgumentException ignored) { }
        }
        throw new NoSuchMethodException(object.getClass() + "." + name);
    }
    static Object make(String name, Object... args) throws Exception {
        for (Constructor<?> ctor : c(name).getConstructors()) {
            try { return ctor.newInstance(args); }
            catch (IllegalArgumentException ignored) { }
        }
        throw new NoSuchMethodException(name);
    }
    static Map<String,Object> obj(Object... fields) {
        Map<String,Object> row = new LinkedHashMap<>();
        for (int i=0; i<fields.length; i+=2) row.put((String)fields[i], fields[i+1]);
        return row;
    }
    static Object rect(Object value) {
        if (value == null) return null;
        Rectangle r = (Rectangle)value;
        return List.of(r.x, r.y, r.width, r.height);
    }
    static Object rational(Object value) { return value == null ? null : value.toString(); }
    static Object field(Object object, String type, String name) throws Exception {
        Field f = c(type).getDeclaredField(name); f.setAccessible(true); return f.get(object);
    }
    static List<Object> names(Collection<?> objects, Map<Object,String> labels) {
        List<Object> result = new ArrayList<>();
        for (Object o : objects) result.add(labels.get(o));
        return result;
    }
    static Object state(Object sig, Object measure, Object stack, List<Object> chords,
                        Map<Object,String> labels, Map<Object,Object> voices,
                        Map<Object,Object> slots, List<Object> relations) throws Exception {
        List<Object> rows=new ArrayList<>(), liveRelations=new ArrayList<>(), voiceRows=new ArrayList<>();
        for (Object ch : chords) {
            boolean attached = call(ch,"getSig") != null;
            Object part=call(ch,"getPart"), staff=call(ch,"getStaff");
            rows.add(obj("label",labels.get(ch),"registered",call(sig,"containsVertex",ch),
                "removed",call(ch,"isRemoved"),"members",names((Collection<?>)call(ch,"getMembers"),labels),
                "stem",attached?labels.get(call(ch,"getStem")):null,
                "bounds",rect(attached?call(ch,"getBounds"):field(ch,"sig.inter.AbstractInter","bounds")),
                "grade",call(ch,"getGrade"),"manual",call(ch,"isManual"),
                "part_id",part==null?null:call(part,"getId"),"staff_id",staff==null?null:call(staff,"getId"),
                "same_measure",call(ch,"getMeasure")==measure,
                "same_voice",call(ch,"getVoice")==voices.get(ch),
                "same_slot",call(ch,"getSlot")==slots.get(ch),
                "voice_id",call(ch,"getVoice")==null?null:call(call(ch,"getVoice"),"getId"),
                "slot_id",call(ch,"getSlot")==null?null:call(call(ch,"getSlot"),"getId"),
                "time_offset",rational(call(ch,"getTimeOffset"))));
        }
        for(Object edge:relations) liveRelations.add(obj("label",labels.get(edge),
            "kind",edge.getClass().getSimpleName(),"registered",call(sig,"containsEdge",edge),
            "manual",call(edge,"isManual")));
        for(Object voice:(Collection<?>)call(measure,"getVoices")) voiceRows.add(obj(
            "id",call(voice,"getId"),"chords",names((Collection<?>)call(voice,"getChords"),labels)));
        return obj("chords",rows,"measure_chords",names((Collection<?>)call(measure,"getHeadChords"),labels),
            "voices",voiceRows,"relations",liveRelations,
            "expected",rational(call(stack,"getExpectedDuration")),
            "actual",rational(call(stack,"getActualDuration")));
    }
    public static void main(String[] args) throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> cases=new ArrayList<>();
        for(String mode:List.of("existing_target", "new_target")) {
            Object book=c("sheet.Book").getMethod("loadBook",Path.class).invoke(null,Path.of(args[0]));
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            Object system=((List<?>)call(sheet,"getSystems")).get(0),sig=call(system,"getSig");
            Object stack=((List<?>)call(system,"getStacks")).get(0);
            call(make("sheet.rhythm.PageRhythm",call(system,"getPage")),"reprocessStack",stack);
            Object measure=((List<?>)call(stack,"getMeasures")).get(0);
            List<Object> chords=new ArrayList<>((Collection<?>)call(measure,"getHeadChords"));
            chords.sort(Comparator.comparingInt(ch->{try{return ((Rectangle)call(ch,"getBounds")).x;}catch(Exception e){throw new RuntimeException(e);}}));
            Map<Object,String> labels=new IdentityHashMap<>();
            Map<Object,Object> voices=new IdentityHashMap<>(),slots=new IdentityHashMap<>();
            List<Object> relations=new ArrayList<>(), relationFacts=new ArrayList<>(), stemFacts=new ArrayList<>();
            for(int i=0;i<chords.size();i++) {
                Object ch=chords.get(i);labels.put(ch,"chord"+i);
                int j=0;for(Object h:(Collection<?>)call(ch,"getMembers"))labels.put(h,"head"+i+"_"+j++);
                labels.put(call(ch,"getStem"),"stem"+i);
                Object s=call(ch,"getStem");
                stemFacts.add(obj("label",labels.get(s),"bounds",rect(call(s,"getBounds"))));
                voices.put(ch,call(ch,"getVoice"));slots.put(ch,call(ch,"getSlot"));
            }
            Object oldChord=chords.get(0),targetChord=chords.get(1);
            Object head=((List<?>)call(oldChord,"getMembers")).get(0),stem=call(targetChord,"getStem");
            Object oldHeadStem=null;
            for(Object e:(Collection<?>)call(sig,"edgesOf",head))if(e.getClass().getSimpleName().equals("HeadStemRelation"))oldHeadStem=e;
            if(mode.equals("new_target")) {
                // Align declared setup with the native retained owner: warm
                // actual Part/Staff getters before the nonextensive removal.
                call(targetChord,"getPart");call(targetChord,"getStaff");
                call(targetChord,"remove",false);
            }
            for(Object e:(Collection<?>)call(sig,"edgeSet")) {
                String k=e.getClass().getSimpleName();
                String sourceLabel=labels.get(call(sig,"getEdgeSource",e));
                if(List.of("Containment","ChordStemRelation","HeadStemRelation").contains(k)
                    && Set.of("chord0","chord1","head0_0","head1_0").contains(sourceLabel==null?"":sourceLabel)) {
                    labels.put(e,"initial"+relations.size());relations.add(e);
                    relationFacts.add(obj("label",labels.get(e),"kind",k,
                        "source",labels.get(call(sig,"getEdgeSource",e)),"target",labels.get(call(sig,"getEdgeTarget",e))));
                }
            }
            Object fresh=make("sig.relation.HeadStemRelation");call(fresh,"setGrade",.7);call(fresh,"setManual",true);
            call(fresh,"setHeadSide",call(oldHeadStem,"getHeadSide"));
            call(fresh,"setExtensionPoint",call(oldHeadStem,"getExtensionPoint"));
            Object pair=make("sig.inter.InterPair",head,stem);
            List<Object> tasks=new ArrayList<>((Collection<?>)call(fresh,"preLink",pair));
            for(Object task:tasks) if(task.getClass().getSimpleName().equals("AdditionTask")) {
                Object ch=call(task,"getInter");labels.put(ch,"new_chord");chords.add(ch);
            }
            tasks.add(make("sig.ui.LinkTask",sig,head,stem,fresh));
            labels.put(fresh,"fresh_head_stem");relations.add(fresh);
            relationFacts.add(obj("label","fresh_head_stem","kind","HeadStemRelation","source",labels.get(head),"target",labels.get(stem)));
            List<Object> taskNames=new ArrayList<>(),trace=new ArrayList<>(),steps=new ArrayList<>();
            for(Object task:tasks)taskNames.add(task.getClass().getSimpleName());
            Field f=Class.forName("org.jgrapht.graph.DefaultListenableGraph").getDeclaredField("graphListeners");
            f.setAccessible(true);List<Object> listeners=(List<Object>)f.get(sig);
            for(int i=0;i<listeners.size();i++) {
                Object delegate=listeners.get(i);if(!delegate.getClass().getSimpleName().equals("SigListener"))continue;
                listeners.set(i,Proxy.newProxyInstance(delegate.getClass().getClassLoader(),new Class[]{Class.forName("org.jgrapht.event.GraphListener")},(p,m,v)->{
                    if(!List.of("edgeAdded","edgeRemoved","vertexAdded","vertexRemoved").contains(m.getName()))return m.invoke(delegate,v);
                    Object entity=call(v[0],m.getName().startsWith("edge")?"getEdge":"getVertex");
                    if(m.getName().startsWith("edge")&&!labels.containsKey(entity)) {
                        labels.put(entity,"created"+relations.size());relations.add(entity);
                        relationFacts.add(obj("label",labels.get(entity),"kind",entity.getClass().getSimpleName(),
                            "source",labels.get(call(v[0],"getEdgeSource")),"target",labels.get(call(v[0],"getEdgeTarget"))));
                    }
                    trace.add(obj("event",m.getName(),"stage","enter","kind",entity.getClass().getSimpleName(),"entity",labels.get(entity),"state",state(sig,measure,stack,chords,labels,voices,slots,relations)));
                    try{return m.invoke(delegate,v);}finally{trace.add(obj("event",m.getName(),"stage","leave","kind",entity.getClass().getSimpleName(),"entity",labels.get(entity),"state",state(sig,measure,stack,chords,labels,voices,slots,relations)));}
                }));
            }
            Object before=state(sig,measure,stack,chords,labels,voices,slots,relations);
            for(String operation:List.of("do","undo","redo")) {
                List<Object> ordered=new ArrayList<>(tasks);if(operation.equals("undo"))Collections.reverse(ordered);
                for(Object task:ordered) {
                    trace.clear();call(task,operation.equals("undo")?"performUndo":"performDo");
                    steps.add(obj("operation",operation,"task",task.getClass().getSimpleName(),
                        "callbacks",new ArrayList<>(trace),"state",state(sig,measure,stack,chords,labels,voices,slots,relations)));
                }
            }
            cases.add(obj("name",mode,"task_names",taskNames,"stem_facts",stemFacts,"relation_facts",relationFacts,"before",before,"steps",steps));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[1]).toFile(),obj(
            "original_source","9e1e55cd2746037d059345881c53e6a6754bffbd",
            "scope","original preLink task list and manual final LinkTask on actual timed saved owners; task and callback intermediates, no automatic discovery/controller epilog/PAGE/persistence claim",
            "cases",cases));System.exit(0);
    }
}
