// SPDX-License-Identifier: AGPL-3.0-or-later
// Original KeyColumn.retrieveKeys, with passive Map-operation checkpoints only.
// No getBestKeyInter/context recomputation is called by observation.
import org.audiveris.omr.sheet.Book;
import java.util.*;import java.nio.file.*;
public class KeyColumnProbe extends KeyBuilderProcessProbe {
    private final Object book;
    private Object system,column;
    private Map<Object,Object> originalBuilders;
    private final List<String> events=new ArrayList<>();
    private boolean recording;
    public KeyColumnProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
    String state()throws Exception {
        List<String> staffs=new ArrayList<>(),parts=new ArrayList<>();
        for(Object part:(List<?>)call(system,"getParts")){
            List<String> ids=new ArrayList<>();for(Object staff:(List<?>)call(part,"getStaves"))ids.add(call(staff,"getId").toString());
            parts.add("{\"id\":"+call(part,"getId")+",\"staff_ids\":["+String.join(",",ids)+"]}");
        }
        for(Object staff:(List<?>)call(system,"getStaves")){
            Object builder=originalBuilders.get(staff),header=call(staff,"getHeader");String builderState="null";
            if(builder!=null){activeBuilder=builder;activeStaff=staff;builderState=snapshot("column_checkpoint");}
            List<String> frozen=new ArrayList<>();Object sig=call(system,"getSig");
            for(Object vertex:(Collection<?>)call(sig,"vertexSet"))if(call(vertex,"getStaff")==staff && (Boolean)call(vertex,"isFrozen"))frozen.add(call(vertex,"getId").toString());
            staffs.add("{\"staff_id\":"+call(staff,"getId")+",\"header_start\":"+call(staff,"getHeaderStart")+",\"header_stop\":"+call(staff,"getHeaderStop")
                +",\"clef_stop\":"+call(staff,"getClefStop")+",\"key_stop\":"+call(staff,"getKeyStop")+",\"header_key\":"+inter(get(header,"key"))
                +",\"header_clef\":"+inter(get(header,"clef"))+",\"alter_starts\":"+get(header,"alterStarts")
                +",\"key_range\":"+(get(header,"keyRange")==null?"null":range(get(header,"keyRange")))+",\"frozen_inter_ids\":["+String.join(",",frozen)+"],\"builder_state\":"+builderState+"}");
        }
        return "{\"global_offsets\":"+get(column,"globalOffsets")+",\"parts\":["+String.join(",",parts)+"],\"staffs\":["+String.join(",",staffs)+"]}";
    }
    void observe(String operation,Object staff){
        if(recording)return;recording=true;
        try{
            List<String> stack=new ArrayList<>();for(StackTraceElement e:Thread.currentThread().getStackTrace())if(e.getClassName().startsWith("org.audiveris.omr.sheet.key."))stack.add(e.getClassName()+"."+e.getMethodName()+":"+e.getLineNumber());
            events.add("{\"index\":"+events.size()+",\"operation\":"+q(operation)+",\"staff_id\":"+(staff==null?"null":call(staff,"getId"))+",\"original_call_stack\":["+String.join(",",stack.stream().map(KeyBuilderProcessProbe::q).toList())+"],\"state\":"+state()+"}");
        }catch(Exception e){throw new RuntimeException(e);}finally{recording=false;}
    }
    class RecordingShapeMap extends AbstractMap<Object,Object>{
        final Map<Object,Object> original;final Object staff;
        RecordingShapeMap(Map<Object,Object> original,Object staff){this.original=original;this.staff=staff;}
        public Set<Entry<Object,Object>> entrySet(){return original.entrySet();}
        public Object get(Object key){observe("before_shape_builder_get_"+key,staff);return original.get(key);}
        public Collection<Object> values(){return original.values();}
    }
    class RecordingBuilders extends AbstractMap<Object,Object>{
        int valuesPass;
        public Set<Entry<Object,Object>> entrySet(){return originalBuilders.entrySet();}
        @SuppressWarnings("unchecked") public Object put(Object key,Object value){try{putShapeMap(value,key);}catch(Exception e){throw new RuntimeException(e);}return originalBuilders.put(key,value);}
        void putShapeMap(Object builder,Object staff)throws Exception{KeyBuilderProcessProbe.put(builder,"shapeBuilders",new RecordingShapeMap((Map<Object,Object>)KeyColumnProbe.get(builder,"shapeBuilders"),staff));}
        public Object get(Object key){observe("before_builders_get",key);return originalBuilders.get(key);}
        public Collection<Object> values(){
            int pass=++valuesPass;observe("before_builders_values_"+pass,null);
            return new AbstractCollection<>(){
                public int size(){return originalBuilders.size();}
                public Iterator<Object> iterator(){Iterator<Entry<Object,Object>> it=originalBuilders.entrySet().iterator();return new Iterator<>(){
                    Object staff;boolean pending;
                    public boolean hasNext(){if(pending){observe("after_builder_iteration_"+pass,staff);pending=false;}return it.hasNext();}
                    public Object next(){Entry<Object,Object> e=it.next();staff=e.getKey();pending=true;return e.getValue();}
                };}
            };
        }
    }
    @SuppressWarnings("unchecked") public void process(){try{
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);
        List<String> results=new ArrayList<>();
        for(Object sys:(List<?>)call(sheet,"getSystems")){
            system=sys;events.clear();
            Object hb=Class.forName("org.audiveris.omr.sheet.header.HeaderBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(system);
            call(hb,"computeHeaderStarts");int clefOffset=(Integer)call(get(hb,"clefColumn"),"retrieveClefs");call(hb,"setSystemClefStop",clefOffset);
            column=get(hb,"keyColumn");originalBuilders=(Map<Object,Object>)get(column,"builders");
            if(!Boolean.getBoolean("native.probe.plainColumn"))put(column,"builders",new RecordingBuilders());
            int width=(Integer)get(hb,"maxHeaderWidth");int offset=(Integer)call(column,"retrieveKeys",width);
            results.add("{\"system_id\":"+call(system,"getId")+",\"multi_staff\":"+call(system,"isMultiStaff")+",\"max_header_width\":"+width+",\"clef_column_offset\":"+clefOffset
                +",\"max_slice_dist\":"+get(get(column,"params"),"maxSliceDist")+",\"returned_key_column_offset\":"+offset+",\"final_state\":"+state()+",\"chronology\":["+String.join(",",events)+"]}");
        }
        Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"stage\":\"original_KeyColumn_retrieveKeys_including_finalizeKey_before_time_column\",\"passive_map_hooks\":"+!Boolean.getBoolean("native.probe.plainColumn")+",\"systems\":["+String.join(",",results)+"]}\n");
    }catch(Exception e){throw new RuntimeException(e);}}
}
