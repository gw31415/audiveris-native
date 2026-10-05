// SPDX-License-Identifier: AGPL-3.0-or-later
// Calls original full KeyBuilder.process; snapshots original FLAT/SHARP iteration
// and final selection, before KeyColumn alignment/finalizeKey. Delegating extractor
// hooks call original super methods and only record extraction boundaries.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;

public class KeyBuilderProcessProbe extends GridFinalProbe {
    private final Object book;
    private final List<String> trailingChecks=new ArrayList<>();
    private final List<String> extractionEvents=new ArrayList<>();
    private final IdentityHashMap<Object,String> lastSliceStates=new IdentityHashMap<>();
    private final List<String> processStages=new ArrayList<>();
    private Object activeBuilder, activeStaff;
    public KeyBuilderProcessProbe(Book b,SortedSet<Integer> ids){super(b,ids);book=b;}
    static void put(Object object,String name,Object value)throws Exception{
        for(Class<?> c=object.getClass();c!=null;c=c.getSuperclass())try{Field f=c.getDeclaredField(name);f.setAccessible(true);f.set(object,value);return;}catch(NoSuchFieldException ignored){}
        throw new NoSuchFieldException(name);
    }
    static String q(Object value){return value==null?"null":"\""+value.toString().replace("\\","\\\\").replace("\"","\\\"")+"\"";}
    static String canonicalGlyph(Object g)throws Exception{return g==null?"null":glyph(g).replaceFirst("\\\"id\\\":-?[0-9]+,","");}
    static String eval(Object e)throws Exception{return e==null?"null":"{\"shape\":"+q(get(e,"shape"))+",\"grade\":"+get(e,"grade")+"}";}
    static String inter(Object value)throws Exception{
        if(value==null)return "null";
        String extra="";String type=value.getClass().getSimpleName();
        if(type.equals("ClefInter"))extra+=",\"clef_kind\":"+q(call(value,"getKind"))+",\"pitch\":"+call(value,"getPitch");
        if(type.equals("KeyAlterInter"))extra+=",\"pitch\":"+call(value,"getPitch")+",\"measured_pitch\":"+call(value,"getMeasuredPitch");
        if(type.equals("KeyInter")){
            List<String> members=new ArrayList<>();for(Object member:(Collection<?>)call(value,"getMembers"))members.add(inter(member));
            extra+=",\"fifths\":"+call(value,"getFifths")+",\"members\":["+String.join(",",members)+"]";
        }
        return "{\"id\":"+call(value,"getId")+",\"kind\":"+q(type)+",\"shape\":"+q(call(value,"getShape"))+",\"grade\":"+call(value,"getGrade")
            +",\"contextual_grade\":"+call(value,"getContextualGrade")+",\"bounds\":"+rect(call(value,"getBounds"))+",\"removed\":"+call(value,"isRemoved")
            +",\"glyph\":"+canonicalGlyph(call(value,"getGlyph"))+extra+"}";
    }
    static String slice(Object item)throws Exception{return "{\"id\":"+call(item,"getId")+",\"rect\":"+rect(call(item,"getRect"))
        +",\"eval\":"+eval(call(item,"getEval"))+",\"glyph\":"+canonicalGlyph(call(item,"getGlyph"))+",\"alter\":"+inter(call(item,"getAlter"))
        +",\"stuffed\":"+call(item,"isStuffed")+"}";}
    static String range(Object range)throws Exception{return "{\"valid\":"+get(range,"valid")+",\"browse_start\":"+get(range,"browseStart")
        +",\"browse_stop\":"+get(range,"browseStop")+",\"start\":"+get(range,"start")+",\"stop\":"+get(range,"stop")+"}";}
    String snapshot(String phase)throws Exception{
        List<String> shapes=new ArrayList<>(),clefs=new ArrayList<>(),sigClefs=new ArrayList<>(),relations=new ArrayList<>();
        for(var entry:((Map<?,?>)get(activeBuilder,"shapeBuilders")).entrySet()){
            Object sb=entry.getValue();List<String> slices=new ArrayList<>();for(Object item:(List<?>)get(sb,"roi"))slices.add(slice(item));
            shapes.add("{\"shape\":"+q(entry.getKey())+",\"range\":"+range(get(sb,"range"))+",\"key_inter\":"+inter(get(sb,"keyInter"))+",\"slices\":["+String.join(",",slices)+"]}");
        }
        for(Object c:(List<?>)get(activeBuilder,"clefs"))clefs.add(inter(c));
        Object sig=call(call(activeStaff,"getSystem"),"getSig");
        Set<Object> relevant=new HashSet<>();
        for(Object vertex:(Collection<?>)call(sig,"vertexSet")){
            if(call(vertex,"getStaff")==activeStaff && (vertex.getClass().getSimpleName().equals("ClefInter") || vertex.getClass().getSimpleName().equals("KeyInter") || vertex.getClass().getSimpleName().equals("KeyAlterInter"))) {
                relevant.add(vertex);if(vertex.getClass().getSimpleName().equals("ClefInter"))sigClefs.add(inter(vertex));
            }
        }
        for(Object edge:(Collection<?>)call(sig,"edgeSet")){
            Object source=call(sig,"getEdgeSource",edge),target=call(sig,"getEdgeTarget",edge);
            if(relevant.contains(source)||relevant.contains(target))relations.add("{\"kind\":"+q(edge.getClass().getSimpleName())+",\"source\":"+call(source,"getId")+",\"target\":"+call(target,"getId")+"}");
        }
        return "{\"phase\":"+q(phase)+",\"builder_clefs\":["+String.join(",",clefs)+"],\"remaining_sig_clefs\":["+String.join(",",sigClefs)
            +"],\"shape_states\":["+String.join(",",shapes)+"],\"relations\":["+String.join(",",relations)+"]}";
    }
    public void trailingObserved(java.awt.Rectangle rect,int maxCumul,int minWidth,boolean result) {
        trailingChecks.add("{\"rect\":"+rect(rect)+",\"max_cumul\":"+maxCumul+",\"min_width\":"+minWidth+",\"result\":"+result+"}");
    }
    public void componentsObserved(Object roi,Object shape){try{
        List<String> slices=new ArrayList<>();for(Object item:(List<?>)roi){String state=slice(item);slices.add(state);lastSliceStates.put(item,state);}
        extractionEvents.add("{\"phase\":\"after_retrieveComponents\",\"shape\":"+q(shape)+",\"slices\":["+String.join(",",slices)+"]}");
    }catch(Exception e){throw new RuntimeException(e);}}
    public String extractionBefore(Object roi,Object item,String phase,double minGrade,boolean crop){try{
        return "{\"phase\":"+q(phase)+",\"slice_index\":"+((List<?>)roi).indexOf(item)+",\"minimum_intrinsic_grade\":"+minGrade+",\"crop_neighbors\":"+crop
            +",\"previous_recorded_state\":"+lastSliceStates.getOrDefault(item,"null")+",\"before_extract\":"+slice(item);
    }catch(Exception e){throw new RuntimeException(e);}}
    public void extractionAfter(Object roi,Object item,String before){try{String state=slice(item);extractionEvents.add(before+",\"after_extract\":"+state+"}");lastSliceStates.put(item,state);}catch(Exception e){throw new RuntimeException(e);}}
    final class RecordingShapes extends AbstractMap<Object,Object>{
        final Map<?,?> original;boolean recorded;
        RecordingShapes(Map<?,?> original){this.original=original;}
        @SuppressWarnings("unchecked") public Set<Entry<Object,Object>> entrySet(){return (Set<Entry<Object,Object>>)(Set<?>)original.entrySet();}
        public Collection<Object> values(){
            if(recorded)return new ArrayList<>(original.values());recorded=true;
            List<Object> values=new ArrayList<>(original.values());
            return new AbstractCollection<>(){
                public int size(){return values.size();}
                public Iterator<Object> iterator(){return new Iterator<>(){
                    int index;Object active;
                    public boolean hasNext(){if(active!=null){try{processStages.add(snapshot("after_"+KeyBuilderProcessProbe.get(active,"keyShape")+".process"));}catch(Exception e){throw new RuntimeException(e);}active=null;}return index<values.size();}
                    public Object next(){if(index>=values.size())throw new NoSuchElementException();return active=values.get(index++);}
                };}
            };
        }
    }
    String pitchWindowControls() throws Exception {
        List<?> clefs=(List<?>)get(activeBuilder,"clefs");
        if(clefs.isEmpty())return "[]";
        Object clef=clefs.get(0);int typical=(Integer)get(get(activeBuilder,"params"),"stdGlyphHeight");
        List<String> records=new ArrayList<>();Class<?> kindClass=Class.forName("org.audiveris.omr.sig.inter.ClefInter$ClefKind"),shapeClass=Class.forName("org.audiveris.omr.glyph.Shape");
        for(var entry:((Map<?,?>)get(activeBuilder,"shapeBuilders")).entrySet()){
            int[] pitches=(int[])Class.forName("org.audiveris.omr.sig.inter.KeyInter").getMethod("getPitches",kindClass,shapeClass).invoke(null,call(clef,"getKind"),entry.getKey());
            double offset=(Double)Class.forName("org.audiveris.omr.sig.inter.AbstractPitchedInter").getMethod("getAreaPitchOffset",shapeClass).invoke(null,entry.getKey());
            for(Object item:(List<?>)get(entry.getValue(),"roi")){
                Object saved=call(item,"getRect");int index=(Integer)call(item,"getId")-1;
                for(int height:new int[]{typical-1,typical,typical+1}){
                    call(item,"setPitchRect",clef,entry.getKey(),height);
                    records.add("{\"controlled_input\":true,\"shape\":"+q(entry.getKey())+",\"clef_kind\":"+q(call(clef,"getKind"))
                        +",\"slice_index\":"+index+",\"typical_height\":"+height+",\"pitch_array_value\":"+pitches[index]
                        +",\"area_pitch_offset\":"+offset+",\"original_rect\":"+rect(saved)+",\"pitch_rect\":"+rect(call(item,"getRect"))+"}");
                    call(item,"setRect",saved);
                }
            }
        }
        return "["+String.join(",",records)+"]";
    }
    public void process(){try{
        Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");call(Class.forName("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);
        Class<?> staffClass=Class.forName("org.audiveris.omr.sheet.Staff"),columnClass=Class.forName("org.audiveris.omr.sheet.key.KeyColumn");
        Constructor<?> ctor=Class.forName("org.audiveris.omr.sheet.key.KeyBuilder").getDeclaredConstructor(columnClass,staffClass,int.class,int.class,int.class,boolean.class);ctor.setAccessible(true);
        List<String> staffResults=new ArrayList<>();
        for(Object system:(List<?>)call(sheet,"getSystems")){
            Object hb=Class.forName("org.audiveris.omr.sheet.header.HeaderBuilder").getConstructor(Class.forName("org.audiveris.omr.sheet.SystemInfo")).newInstance(system);
            call(hb,"computeHeaderStarts");int clefOffset=(Integer)call(get(hb,"clefColumn"),"retrieveClefs");call(hb,"setSystemClefStop",clefOffset);
            int width=(Integer)get(hb,"maxHeaderWidth");Object column=get(hb,"keyColumn");
            for(Object staff:(List<?>)call(system,"getStaves")){
                activeStaff=staff;int start=(Integer)call(staff,"getHeaderStart");Integer clefStop=(Integer)call(staff,"getClefStop");int browseStart=(clefStop!=null?clefStop:(Integer)call(staff,"getHeaderStop"))+1;
                activeBuilder=ctor.newInstance(column,staff,width,start,browseStart,true);extractionEvents.clear();trailingChecks.clear();processStages.clear();lastSliceStates.clear();
                put(activeBuilder,"shapeBuilders",new RecordingShapes((Map<?,?>)get(activeBuilder,"shapeBuilders")));
                boolean hooks=!Boolean.getBoolean("native.probe.plainExtractor");
                if(hooks)put(activeBuilder,"extractor",Class.forName("ObservedKeyExtractor").getConstructor(staffClass,KeyBuilderProcessProbe.class).newInstance(staff,this));
                call(activeBuilder,"process");processStages.add(snapshot("after_selectBestClef"));
                staffResults.add("{\"staff_id\":"+call(staff,"getId")+",\"system_id\":"+call(system,"getId")+",\"typical_glyph_height\":"+get(get(activeBuilder,"params"),"stdGlyphHeight")
                    +",\"pitch_window_controls\":"+(Boolean.getBoolean("native.probe.pitchWindowControl")?pitchWindowControls():"[]")+",\"stages\":["+String.join(",",processStages)+"],\"trailing_checks\":["+String.join(",",trailingChecks)+"],\"extraction_events\":["+String.join(",",extractionEvents)+"]}");
            }
        }
        Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"stage\":\"full_per_staff_KeyBuilder_process_before_KeyColumn_alignment_and_finalizeKey\",\"delegating_extractor_hooks\":"+!Boolean.getBoolean("native.probe.plainExtractor")+",\"staffs\":["+String.join(",",staffResults)+"]}\n");
    }catch(Exception e){throw new RuntimeException(e);}}
}
