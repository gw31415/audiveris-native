// SPDX-License-Identifier: AGPL-3.0-or-later
import java.nio.file.Path;
import java.util.*;
import com.fasterxml.jackson.databind.ObjectMapper;
public class TupletFontProbe extends HeadStemPreLinkProbe {
    public static void main(String[] args)throws Exception {
        c("ui.symbol.MusicFont").getMethod("populateAllSymbols").invoke(null);
        List<Object> rows=new ArrayList<>();
        for(Object family:c("ui.symbol.MusicFamily").getEnumConstants())for(int interline=1;interline<=128;interline++)for(String name:List.of("TUPLET_THREE","TUPLET_SIX")){
            Object shape=c("glyph.Shape").getField(name).get(null);Object fs=call(shape,"getFontSymbolByInterline",family,interline);java.awt.Dimension dim=(java.awt.Dimension)call(fs,"getDimension");Object font=fs.getClass().getField("font").get(fs);Object layout=call(fs,"getLayout");java.awt.geom.Rectangle2D bounds=(java.awt.geom.Rectangle2D)call(layout,"getBounds");
            rows.add(obj("family",family.toString(),"interline",interline,"shape",name,"dimension",List.of(dim.width,dim.height),"layout_bounds",List.of(bounds.getX(),bounds.getY(),bounds.getWidth(),bounds.getHeight()),"font_name",call(font,"getName"),"point_size",call(font,"getSize")));
        }
        new ObjectMapper().writerWithDefaultPrettyPrinter().writeValue(Path.of(args[0]).toFile(),obj("scope","untouched pinned FontSymbol base-interline dimension resources for all six MusicFamily values, TWO tuplet shapes, scales1..128; no native outputs/recognition/archive/whole profile","rows",rows));
    }
}
