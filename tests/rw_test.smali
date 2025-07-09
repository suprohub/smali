.class public final Lcom/example/TestClass;
.super Ljava/lang/Object;
.source "TestClass.java"

# interfaces
.implements Ljava/lang/Runnable;

# annotations
.annotation system Ldalvik/annotation/Signature;
    value = {
        "Ljava/lang/Object;",
        "Ljava/lang/Runnable;"
    }
.end annotation

.annotation runtime Lcom/example/ClassAnnotation;
    id = 0x1
    value = "class"
.end annotation

# static fields
.field public static final CONSTANT_FIELD:I = 0x42
    .annotation build Lcom/example/FieldAnnotation;
        name = "constant"
    .end annotation
.end field


# direct methods
.method public constructor <init>()V
    .registers 1

    .line 5
    invoke-direct {p0}, Ljava/lang/Object;-><init>()V

    return-void
.end method

.method public static testMethod(Ljava/lang/String;I)V
    .param p0              # Ljava/lang/String;
        .annotation build Lcom/example/ParamAnnotation;
            required = true
        .end annotation
    .end param
    .param p1, "param2"    # I
    .annotation build Lcom/example/MethodAnnotation;
        version = 0x2
    .end annotation

    .annotation system Ldalvik/annotation/Throws;
        value = {
            Ljava/lang/Exception;
        }
    .end annotation

    .annotation runtime Lorg/checkerframework/checker/nullness/qual/EnsuresNonNull$List;
        value = {
            .subannotation Lorg/checkerframework/checker/nullness/qual/EnsuresNonNull;
                value = {
                    "this.preferences"
                }
            .end subannotation,
            .subannotation Lorg/checkerframework/checker/nullness/qual/EnsuresNonNull;
                value = {
                    "this.monitoringSample"
                }
            .end subannotation
        }
    .end annotation

    .prologue
    .line 10
    const-string v0, "Start"

    invoke-static {v0}, Landroid/util/Log;->d(Ljava/lang/String;Ljava/lang/String;)I

    .line 12
    :try_start_0
    invoke-virtual {p0}, Ljava/lang/String;->length()I
    :try_end_0
    .catch Ljava/lang/Exception; {:try_start_0 .. :try_end_0} :catch_0

    .line 17
    :goto_0
    :try_start_1
    new-instance v0, Ljava/lang/Exception;

    invoke-direct {v0}, Ljava/lang/Exception;-><init>()V

    throw v0
    :try_end_1
    .catchall {:try_start_1 .. :try_end_1} :catchall_0

    .line 19
    :catchall_0
    move-exception v0

    .line 22
    :pswitch_0
    sparse-switch p1, :sswitch_data_0

    .line 29
    :goto_1
    return-void

    .line 13
    :catch_0
    move-exception v0

    .line 14
    .local v0, "e":Ljava/lang/Exception;
    invoke-virtual {v0}, Ljava/lang/Exception;->printStackTrace()V

    goto :goto_0

    .line 24
    :sswitch_0
    nop

    .line 25
    goto :goto_1

    .line 27
    :sswitch_1
    nop

    goto :goto_1

    .line 22
    :sswitch_data_0
    .sparse-switch
        0x0 -> :sswitch_0
        0x1 -> :sswitch_1
    .end sparse-switch

    .line 31
    :array_0
    .array-data 4
        0x1
        0x2
        0x3
    .end array-data
.end method