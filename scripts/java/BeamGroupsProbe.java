// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only uninterrupted original extension, late hooks and group
// formation. Stops before multiple rests, epilog and rhythm processing.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;
public class BeamGroupsProbe extends BeamsFaithfulProbe {
    public BeamGroupsProbe(Book book, SortedSet<Integer> ids) { super(book, ids); }
    static void put(Object object, String name, Object value) throws Exception {
        for (Class<?> type=object.getClass(); type!=null; type=type.getSuperclass()) {
            try { Field f=type.getDeclaredField(name); f.setAccessible(true); f.set(object,value); return; }
            catch (NoSuchFieldException ignored) { }
        }
        throw new NoSuchFieldException(name);
    }
    static String groupSnapshot(Object sys)throws Exception{Object sig=call(sys,"getSig");List<String> groups=new ArrayList<>(),supports=new ArrayList<>();for(Object g:(List<?>)sig.getClass().getMethod("inters",Class.class).invoke(sig,cls("org.audiveris.omr.sig.inter.BeamGroupInter"))){List<String> members=new ArrayList<>();for(Object b:(List<?>)call(g,"getMembers"))members.add(beam(b));groups.add("{\"bounds\":"+groupRect((java.awt.Rectangle)call(g,"getBounds"))+",\"members\":["+String.join(",",members)+"]}");}for(Object e:(Set<?>)call(sig,"edgeSet"))if(e.getClass().getSimpleName().equals("BeamBeamRelation"))supports.add("{\"source\":"+beam(call(sig,"getEdgeSource",e))+",\"target\":"+beam(call(sig,"getEdgeTarget",e))+"}");return "{\"groups\":["+String.join(",",groups)+"],\"supports\":["+String.join(",",supports)+"]}";}
    static String groupRect(java.awt.Rectangle r){return "["+r.x+","+r.y+","+r.width+","+r.height+"]";}
    @SuppressWarnings({"unchecked", "rawtypes"})
    public void process() {
        try {
            Object sheet=call(((List<?>)call(book,"getStubs")).get(0),"getSheet");
            call(cls("org.audiveris.omr.sheet.grid.GridStep").getConstructor().newInstance(),"doit",sheet);
            for (Object sys:(List<?>)call(sheet,"getSystems"))
                call(construct("org.audiveris.omr.sheet.header.HeaderBuilder",new Class<?>[]{cls("org.audiveris.omr.sheet.SystemInfo")},sys),"processHeader");
            call(cls("org.audiveris.omr.sheet.stem.StemSeedsStep").getConstructor().newInstance(),"doit",sheet);
            Object step=cls("org.audiveris.omr.sheet.beam.BeamsStep").getConstructor().newInstance();
            Object context=call(step,"doProlog",sheet);
            Object scale=call(sheet,"getScale"), picture=call(sheet,"getPicture");
            Object noStaff=call(picture,"getSource",enumValue("org.audiveris.omr.sheet.Picture$SourceKey","NO_STAFF"));
            int beam=(Integer)call(scale,"getItemValue",enumValue("org.audiveris.omr.sheet.Scale$Item","beam"));
            Integer small=(Integer)call(scale,"getItemValue",enumValue("org.audiveris.omr.sheet.Scale$Item","smallBeam"));
            boolean smallSwitch=(Boolean)call(call(call(sheet,"getStub"),"getProcessingSwitches"),"getValue",enumValue("org.audiveris.omr.sheet.ProcessingSwitch","smallBeams"));
            if (small==null && smallSwitch)
                small=(int)Math.rint(beam*(Double)cls("org.audiveris.omr.sheet.beam.BeamsBuilder").getMethod("getCueBeamRatio").invoke(null));
            Object group=enumValue("org.audiveris.omr.glyph.GlyphGroup","BEAM_SPOT");
            List<String> records=new ArrayList<>();
            for (Object sys:(List<?>)call(sheet,"getSystems")) {
                Object builder=construct("org.audiveris.omr.sheet.beam.BeamsBuilder",new Class<?>[]{cls("org.audiveris.omr.sheet.SystemInfo"),cls("org.audiveris.omr.lag.Lag")},sys,get(context,"spotLag"));
                Object standard=construct("org.audiveris.omr.sheet.beam.BeamsBuilder$ItemParameters",new Class<?>[]{cls("org.audiveris.omr.sheet.Scale"),double.class,boolean.class},scale,(double)beam,false);
                Object smallParams=small==null?null:construct("org.audiveris.omr.sheet.beam.BeamsBuilder$ItemParameters",new Class<?>[]{cls("org.audiveris.omr.sheet.Scale"),double.class,boolean.class},scale,(double)small,true);
                put(builder,"stdParams",standard); put(builder,"smallParams",smallParams); put(builder,"pixelFilter",noStaff);
                List spots=(List)call(sys,"getGroupedGlyphs",group);
                spots.sort((Comparator)cls("org.audiveris.omr.glyph.Glyphs").getField("byFullOrdinate").get(null));
                put(builder,"sortedBeamSpots",spots);
                String ordered=spots(spots);
                call(builder,"createBeams");
                String initial=finalSystem(sys),assigned=spots((Collection<?>)get(builder,"assignedSpots"));
                spots.removeAll((Collection)get(builder,"assignedSpots"));
                call(builder,"extendBeams");
                String extended=finalSystem(sys); spots.removeAll((Collection)get(builder,"assignedSpots")); call(builder,"buildHooks"); String hooked=finalSystem(sys); cls("org.audiveris.omr.sig.inter.BeamGroupInter").getMethod("populateSystem",cls("org.audiveris.omr.sheet.SystemInfo")).invoke(null,sys); String grouped=groupSnapshot(sys); List<String> raw=new ArrayList<>(); for(Object v:(List<?>)get(builder,"rawSystemBeams")) raw.add("{\"removed\":"+call(v,"isRemoved")+",\"beam\":"+beam(v)+"}");
                records.add("{\"groups\":"+grouped+",\"system_id\":"+call(sys,"getId")+",\"ordered_spots\":"+ordered+",\"assigned_spots\":"+assigned+",\"after_create_beams\":"+initial+",\"after_extend_beams\":"+extended+",\"after_late_hooks\":"+hooked+",\"raw_after\":["+String.join(",",raw)+"],\"standard_parameters\":"+fields(standard)+",\"small_parameters\":"+fields(smallParams)+",\"global_parameters\":"+fields(get(builder,"params"))+"}");
            }
            // Stop after group formation; no rests or epilog.
            List<String> finals=new ArrayList<>();
            for (Object sys:(List<?>)call(sheet,"getSystems")) finals.add(finalSystem(sys));
            String finalJson="{\"head_scale\":"+headScale(scale)+",\"head_spots_mask\":"+mask(call(call(picture,"getVerticalTable",enumValue("org.audiveris.omr.sheet.Picture$TableKey","HEAD_SPOTS")),"getBuffer"))+",\"no_staff_after\":"+mask(noStaff)+",\"systems\":["+String.join(",",finals)+"]}";
            Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"stage\":\"original_uninterrupted_extension_hooks_populateSystem_before_rests\",\"initialization\":\"private fields initialized with original constructors and comparator in original buildBeams order\",\"native_outputs_used\":false,\"no_staff\":"+mask(noStaff)+",\"systems\":["+String.join(",",records)+"],\"final\":"+finalJson+"}\n");
        } catch (Exception error) { throw new RuntimeException(error); }
    }
}
