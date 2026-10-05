// SPDX-License-Identifier: AGPL-3.0-or-later
// Development-only original createBeams boundary, followed by the remaining
// original builder calls so the final state can be cross-checked against doit.
import org.audiveris.omr.sheet.Book;
import java.util.*;
import java.nio.file.*;
import java.lang.reflect.*;
public class BeamLateHooksProbe extends BeamsFaithfulProbe {
    public BeamLateHooksProbe(Book book, SortedSet<Integer> ids) { super(book, ids); }
    static void put(Object object, String name, Object value) throws Exception {
        for (Class<?> type=object.getClass(); type!=null; type=type.getSuperclass()) {
            try { Field f=type.getDeclaredField(name); f.setAccessible(true); f.set(object,value); return; }
            catch (NoSuchFieldException ignored) { }
        }
        throw new NoSuchFieldException(name);
    }
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
                String extended=finalSystem(sys); spots.removeAll((Collection)get(builder,"assignedSpots")); call(builder,"buildHooks"); String hooked=finalSystem(sys); List<String> raw=new ArrayList<>(); for(Object v:(List<?>)get(builder,"rawSystemBeams")) raw.add("{\"removed\":"+call(v,"isRemoved")+",\"beam\":"+beam(v)+"}");
                records.add("{\"system_id\":"+call(sys,"getId")+",\"ordered_spots\":"+ordered+",\"assigned_spots\":"+assigned+",\"after_create_beams\":"+initial+",\"after_extend_beams\":"+extended+",\"after_late_hooks\":"+hooked+",\"raw_after\":["+String.join(",",raw)+"],\"standard_parameters\":"+fields(standard)+",\"small_parameters\":"+fields(smallParams)+",\"global_parameters\":"+fields(get(builder,"params"))+"}");
            }
            // Stop at extension boundary; no hooks, groups, rests or epilog.
            List<String> finals=new ArrayList<>();
            for (Object sys:(List<?>)call(sheet,"getSystems")) finals.add(finalSystem(sys));
            String finalJson="{\"head_scale\":"+headScale(scale)+",\"head_spots_mask\":"+mask(call(call(picture,"getVerticalTable",enumValue("org.audiveris.omr.sheet.Picture$TableKey","HEAD_SPOTS")),"getBuffer"))+",\"no_staff_after\":"+mask(noStaff)+",\"systems\":["+String.join(",",finals)+"]}";
            Files.writeString(Path.of(System.getProperty("native.probe.output")),"{\"stage\":\"original_uninterrupted_extendBeams_then_buildHooks_before_groups_rests\",\"initialization\":\"private fields initialized with original constructors and comparator in original buildBeams order\",\"native_outputs_used\":false,\"no_staff\":"+mask(noStaff)+",\"systems\":["+String.join(",",records)+"],\"final\":"+finalJson+"}\n");
        } catch (Exception error) { throw new RuntimeException(error); }
    }
}
