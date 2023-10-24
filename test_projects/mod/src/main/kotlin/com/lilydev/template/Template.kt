package com.lilydev.template

import net.fabricmc.api.ModInitializer
import org.slf4j.Logger
import org.slf4j.LoggerFactory

object Template : ModInitializer {

    const val MOD_ID: String = "template"
    const val MOD_NAME: String = "Template Mod"

    @JvmField
    val LOGGER: Logger = LoggerFactory.getLogger(MOD_NAME)

    override fun onInitialize() {
        LOGGER.info("Hello Fabric world from $MOD_NAME")
    }
}