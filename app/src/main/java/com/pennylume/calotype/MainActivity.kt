package com.pennylume.calotype

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.viewModels
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.safeDrawingPadding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

private val Ink = Color(0xFF0B0B0C)
private val Amber = Color(0xFFE8B04A)
private val Muted = Color(0xFFB8B8BE)

class MainActivity : ComponentActivity() {
    private val vm: PreviewViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        setContent {
            MaterialTheme(colorScheme = darkColorScheme(primary = Amber, onPrimary = Ink, background = Ink, surface = Ink, onBackground = Color.White, onSurface = Color.White)) {
                PreviewScreen(vm)
            }
        }
    }
}

@Composable
private fun PreviewScreen(vm: PreviewViewModel) {
    // Previews render at screen size, never full resolution (memory budget).
    val config = LocalConfiguration.current
    val longEdge = with(LocalDensity.current) { maxOf(config.screenWidthDp, config.screenHeightDp).dp.roundToPx() }.coerceIn(512, 2048)
    val pick = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        if (uri != null) vm.open(uri, longEdge)
    }
    val state = vm.state

    Column(Modifier.fillMaxSize().background(Ink).safeDrawingPadding().padding(16.dp)) {
        Text("Calotype", color = Color.White, fontSize = 22.sp, fontWeight = FontWeight.SemiBold)
        Box(Modifier.weight(1f).fillMaxWidth().padding(vertical = 16.dp), contentAlignment = Alignment.Center) {
            when (state) {
                PreviewState.Empty -> Message("Open a raw photo to develop it.", "DNG and most camera raws. Nothing leaves your phone.")
                is PreviewState.Developing -> Column(horizontalAlignment = Alignment.CenterHorizontally) {
                    CircularProgressIndicator(color = Amber, modifier = Modifier.size(40.dp))
                    Spacer(Modifier.height(16.dp))
                    Text("Developing ${state.name}…", color = Color.White)
                }
                is PreviewState.Shown -> Image(state.image, contentDescription = "Preview of ${state.name}", contentScale = ContentScale.Fit, modifier = Modifier.fillMaxSize())
                is PreviewState.Failed -> Message("Couldn't open ${state.name}", state.reason)
            }
        }
        if (state is PreviewState.Shown) {
            Text(state.name, color = Color.White, fontWeight = FontWeight.Medium)
            Text(state.detail, color = Muted, fontSize = 13.sp)
            Spacer(Modifier.height(12.dp))
        }
        Button(
            onClick = { pick.launch(arrayOf("*/*")) },
            enabled = state !is PreviewState.Developing,
            colors = ButtonDefaults.buttonColors(containerColor = Amber, contentColor = Ink),
            modifier = Modifier.fillMaxWidth().height(56.dp),
        ) { Text("Open raw photo", fontWeight = FontWeight.SemiBold, fontSize = 16.sp) }
    }
}

@Composable
private fun Message(title: String, body: String) {
    Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        Text(title, color = Color.White, fontSize = 18.sp, fontWeight = FontWeight.Medium, textAlign = TextAlign.Center)
        Text(body, color = Muted, textAlign = TextAlign.Center)
    }
}
