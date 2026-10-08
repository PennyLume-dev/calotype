package com.pennylume.calotype

import android.app.Application
import android.graphics.BitmapFactory
import android.net.Uri
import android.os.SystemClock
import android.provider.OpenableColumns
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import com.pennylume.calotype.bridge.BridgeException
import com.pennylume.calotype.bridge.Editor
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import java.io.File

sealed interface PreviewState {
    data object Empty : PreviewState
    data class Developing(val name: String) : PreviewState
    data class Shown(val image: ImageBitmap, val name: String, val detail: String) : PreviewState
    data class Failed(val name: String, val reason: String) : PreviewState
}

class PreviewViewModel(app: Application) : AndroidViewModel(app) {
    var state by mutableStateOf<PreviewState>(PreviewState.Empty)
        private set

    // One engine session for the app's lifetime; created on first use, off the main thread.
    private val editor by lazy { Editor() }

    fun open(uri: Uri, longEdge: Int) {
        if (state is PreviewState.Developing) return
        viewModelScope.launch {
            val name = displayName(uri)
            state = PreviewState.Developing(name)
            state = withContext(Dispatchers.IO) { develop(uri, name, longEdge) }
        }
    }

    private fun develop(uri: Uri, name: String, longEdge: Int): PreviewState = try {
        val file = copyIn(uri, name)
        val t0 = SystemClock.elapsedRealtime()
        val id = editor.import(file.path)
        val r = editor.preview(id, longEdge.toUInt())
        val ms = SystemClock.elapsedRealtime() - t0
        val bitmap = BitmapFactory.decodeByteArray(r.jpeg, 0, r.jpeg.size)
            ?: throw IllegalStateException("preview could not be decoded")
        PreviewState.Shown(bitmap.asImageBitmap(), name, "${r.width} × ${r.height} · developed in $ms ms")
    } catch (e: BridgeException.Failed) {
        PreviewState.Failed(name, e.reason)
    } catch (e: Exception) {
        PreviewState.Failed(name, e.message ?: e.javaClass.simpleName)
    }

    /** The engine reads plain paths, so picked files are copied into the app's own storage. */
    private fun copyIn(uri: Uri, name: String): File {
        val dir = File(getApplication<Application>().cacheDir, "import").apply { mkdirs() }
        val out = File(dir, name.replace(Regex("[^A-Za-z0-9._-]"), "_").ifEmpty { "photo" })
        val input = getApplication<Application>().contentResolver.openInputStream(uri)
            ?: throw IllegalStateException("could not open the file")
        input.use { src -> out.outputStream().use { src.copyTo(it) } }
        return out
    }

    private fun displayName(uri: Uri): String =
        getApplication<Application>().contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)
            ?.use { c -> if (c.moveToFirst()) c.getString(0) else null }
            ?: uri.lastPathSegment ?: "photo"
}
